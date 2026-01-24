use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use teloxide::prelude::*;
use diesel::prelude::*;
use rig::completion::Prompt;
use rig::providers::openai;
use rig::client::ProviderClient;
use rig::client::CompletionClient;

use crate::database::DbPool;
use crate::models::{Agent, TelegramConfig};
use crate::schema;

pub type BotShutdownSender = mpsc::Sender<()>;

pub struct TelegramBotManager {
    pub db_pool: DbPool,
    pub running_bots: Arc<RwLock<HashMap<String, BotShutdownSender>>>,
}

impl TelegramBotManager {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            running_bots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_bot(&self, config_id: &str) -> Result<(), String> {
        let config = self.get_config(config_id)?;

        if config.is_active == 0 {
            return Err("Bot configuration is not active".to_string());
        }

        // Check if bot is already running
        {
            let bots = self.running_bots.read().await;
            if bots.contains_key(config_id) {
                return Err("Bot is already running".to_string());
            }
        }

        let agent = self.get_agent(&config.agent_id)?;
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // Parse allowed chat IDs
        let allowed_chats: Option<Vec<i64>> = config.allowed_chat_ids.as_ref().map(|ids| {
            ids.split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect()
        });

        let bot = Bot::new(&config.bot_token);
        let config_id_owned = config_id.to_string();
        let running_bots = self.running_bots.clone();
        let db_pool = self.db_pool.clone();

        tokio::spawn(async move {
            let handler = Update::filter_message()
                .filter_map(|msg: Message| {
                    let text = msg.text()?.to_string();
                    Some((msg, text))
                })
                .endpoint(move |bot: Bot, (msg, text): (Message, String)| {
                    let agent = agent.clone();
                    let allowed_chats = allowed_chats.clone();
                    let _db_pool = db_pool.clone(); // Reserved for future use (storing messages)

                    async move {
                        // Check if chat is allowed (if restrictions exist)
                        if let Some(ref allowed) = allowed_chats {
                            if !allowed.contains(&msg.chat.id.0) {
                                log::info!("Ignoring message from unauthorized chat: {}", msg.chat.id.0);
                                return Ok::<(), teloxide::RequestError>(());
                            }
                        }

                        // Send typing indicator
                        let _ = bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing).await;

                        // Process message with agent
                        match process_message_with_agent(&agent, &text).await {
                            Ok(response) => {
                                // Split long messages (Telegram has 4096 char limit)
                                for chunk in split_message(&response, 4000) {
                                    if let Err(e) = bot.send_message(msg.chat.id, chunk).await {
                                        log::error!("Failed to send message: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Sorry, I encountered an error: {}", e);
                                let _ = bot.send_message(msg.chat.id, error_msg).await;
                            }
                        }

                        Ok(())
                    }
                });

            let mut dispatcher = Dispatcher::builder(bot, handler)
                .enable_ctrlc_handler()
                .build();

            let shutdown_token = dispatcher.shutdown_token();

            tokio::select! {
                _ = dispatcher.dispatch() => {
                    log::info!("Bot dispatcher stopped");
                }
                _ = shutdown_rx.recv() => {
                    log::info!("Received shutdown signal for bot");
                    shutdown_token.shutdown().ok();
                }
            }

            // Clean up when done
            let mut bots = running_bots.write().await;
            bots.remove(&config_id_owned);
            log::info!("Bot {} has been stopped", config_id_owned);
        });

        // Store the shutdown sender
        {
            let mut bots = self.running_bots.write().await;
            bots.insert(config_id.to_string(), shutdown_tx);
        }

        log::info!("Started Telegram bot for config: {}", config_id);
        Ok(())
    }

    pub async fn stop_bot(&self, config_id: &str) -> Result<(), String> {
        let shutdown_tx = {
            let mut bots = self.running_bots.write().await;
            bots.remove(config_id)
        };

        match shutdown_tx {
            Some(tx) => {
                let _ = tx.send(()).await;
                Ok(())
            }
            None => Err("Bot is not running".to_string()),
        }
    }

    pub async fn stop_all_bots(&self) {
        let bots: Vec<(String, BotShutdownSender)> = {
            let mut running = self.running_bots.write().await;
            running.drain().collect()
        };

        for (id, tx) in bots {
            log::info!("Stopping bot: {}", id);
            let _ = tx.send(()).await;
        }
    }

    pub async fn is_bot_running(&self, config_id: &str) -> bool {
        let bots = self.running_bots.read().await;
        bots.contains_key(config_id)
    }

    pub async fn get_running_bot_ids(&self) -> Vec<String> {
        let bots = self.running_bots.read().await;
        bots.keys().cloned().collect()
    }

    fn get_config(&self, config_id: &str) -> Result<TelegramConfig, String> {
        use schema::telegram_configs::dsl::*;

        let mut conn = self.db_pool.get().map_err(|e| e.to_string())?;
        telegram_configs
            .filter(id.eq(config_id))
            .first::<TelegramConfig>(&mut conn)
            .map_err(|e| format!("Config not found: {}", e))
    }

    fn get_agent(&self, agent_id: &str) -> Result<Agent, String> {
        use schema::agents::dsl::*;

        let mut conn = self.db_pool.get().map_err(|e| e.to_string())?;
        agents
            .filter(id.eq(agent_id))
            .first::<Agent>(&mut conn)
            .map_err(|e| format!("Agent not found: {}", e))
    }

    pub async fn start_all_active_bots(&self) -> Result<(), String> {
        let configs = self.get_active_configs()?;

        for config in configs {
            if let Err(e) = self.start_bot(&config.id).await {
                log::error!("Failed to start bot {}: {}", config.id, e);
            }
        }

        Ok(())
    }

    fn get_active_configs(&self) -> Result<Vec<TelegramConfig>, String> {
        use schema::telegram_configs::dsl::*;

        let mut conn = self.db_pool.get().map_err(|e| e.to_string())?;
        telegram_configs
            .filter(is_active.eq(1))
            .load::<TelegramConfig>(&mut conn)
            .map_err(|e| e.to_string())
    }
}

async fn process_message_with_agent(agent: &Agent, user_message: &str) -> Result<String, String> {
    let client = openai::Client::from_env();

    // Build the agent with system prompt (preamble)
    let ai_agent = if let Some(prompt) = &agent.system_prompt {
        if !prompt.is_empty() {
            client
                .agent("gpt-4")
                .preamble(prompt)
                .build()
        } else {
            client.agent("gpt-4").build()
        }
    } else {
        client.agent("gpt-4").build()
    };

    ai_agent
        .prompt(user_message)
        .await
        .map_err(|e| format!("AI error: {}", e))
}

fn split_message(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if current.len() + line.len() + 1 > max_len {
            if !current.is_empty() {
                chunks.push(current);

            }
            // If single line is too long, split it
            if line.len() > max_len {
                let mut remaining = line;
                while remaining.len() > max_len {
                    chunks.push(remaining[..max_len].to_string());
                    remaining = &remaining[max_len..];
                }
                current = remaining.to_string();
            } else {
                current = line.to_string();
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_message_short() {
        let text = "Hello world";
        let chunks = split_message(text, 50);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    fn test_split_message_long_no_newlines() {
        let text = "a".repeat(10);
        let chunks = split_message(&text, 5);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "aaaaa");
        assert_eq!(chunks[1], "aaaaa");
    }

    #[test]
    fn test_split_message_with_newlines() {
        let text = "line1\nline2\nline3";
        let chunks = split_message(text, 10);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "line1");
        assert_eq!(chunks[1], "line2");
        assert_eq!(chunks[2], "line3");
    }

    #[test]
    fn test_split_message_complex() {
        let text = "short\nvery long line that needs splitting\nshort";
        let chunks = split_message(text, 10);
        // "short" (5) -> fits
        // "very long " (10) -> fit in next chunk?? No, logic is:
        // current="short"
        // next="very long..." -> 5+1+33 = 39 > 10.
        // push "short". current="very long..."
        // "very long..." > 10. split.
        // push "very long "
        // remaining "line that..."
        
        // Let's trace the actual logic in `split_message`:
        // default max_len 4000. 
        // Here max_len=10.
        
        // Line 1: "short". current="short"
        // Line 2: "very long line...". len=33. 
        // current.len() + line.len() + 1 = 5 + 33 + 1 = 39 > 10.
        // chunks.push("short")
        // line > 10? Yes.
        // split "very long " (10). remain "line that ..."
        // split "line that " (10). remain "needs spli..."
        // split "needs spli" (10). remain "tting"
        // current = "tting"
        // Line 3: "short".
        // current.len() + line.len() + 1 = 5 + 5 + 1 = 11 > 10.
        // chunks.push("tting")
        // current = "short"
        // End. push "short"
        
        // Expected chunks:
        // "short"
        // "very long "
        // "line that "
        // "needs spli"
        // "tting"
        // "short"
        
        assert_eq!(chunks.len(), 6);
        assert_eq!(chunks[0], "short");
        assert_eq!(chunks[1], "very long ");
        assert_eq!(chunks[2], "line that ");
        assert_eq!(chunks[3], "needs spli");
        assert_eq!(chunks[4], "tting");
        assert_eq!(chunks[5], "short");
    }
}
