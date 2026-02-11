//! Unified LLM Provider abstraction
//!
//! This module provides a unified interface for interacting with different LLM providers
//! (OpenAI, Gemini, Anthropic) without repetitive branching throughout the codebase.

use futures::StreamExt;
use log::error;
use rig::agent::MultiTurnStreamItem;
use rig::client::CompletionClient;
use rig::client::ProviderClient;
use rig::completion::{Chat, Message, Prompt};
use rig::providers::{anthropic, gemini, openai};
use rig::streaming::{StreamedAssistantContent, StreamingChat, StreamingPrompt};

/// A token from the LLM stream
#[derive(Debug, Clone)]
pub enum StreamToken {
    Text(String),
    Done,
    Error(String),
}

/// Unified LLM client that abstracts provider differences
pub struct LlmClient {
    provider: String,
    model: String,
    preamble: Option<String>,
    api_key: Option<String>,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            preamble: None,
            api_key: None,
        }
    }

    /// Set the system prompt/preamble
    pub fn with_preamble(mut self, preamble: &str) -> Self {
        self.preamble = Some(preamble.to_string());
        self
    }

    /// Set the API key explicitly
    pub fn with_api_key(mut self, api_key: &str) -> Self {
        if !api_key.is_empty() {
            self.api_key = Some(api_key.to_string());
        }
        self
    }

    /// Check if the required API key is set
    pub fn check_api_key(&self) -> Result<(), String> {
        if self.api_key.is_some() {
            return Ok(());
        }

        let key_name = match self.provider.as_str() {
            "openai" => "OPENAI_API_KEY",
            "gemini" => "GEMINI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => return Err(format!("Unsupported provider: {}", self.provider)),
        };

        if std::env::var(key_name).unwrap_or_default().is_empty() {
            return Err(format!("Error: {} not set (env or settings)", key_name));
        }
        Ok(())
    }

    /// Simple prompt without history (non-streaming)
    pub async fn prompt(&self, message: &str) -> Result<String, String> {
        self.check_api_key()?;
        let preamble = self.preamble.clone().unwrap_or_default();

        match self.provider.as_str() {
            "openai" => {
                let client = if let Some(key) = &self.api_key {
                    openai::Client::new(key)
                } else {
                    Ok(openai::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent.prompt(message).await.map_err(|e| e.to_string())
            }
            "gemini" => {
                let client = if let Some(key) = &self.api_key {
                    gemini::Client::new(key)
                } else {
                    Ok(gemini::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent.prompt(message).await.map_err(|e| e.to_string())
            }
            "anthropic" => {
                let client = if let Some(key) = &self.api_key {
                    anthropic::Client::new(key)
                } else {
                    Ok(anthropic::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent.prompt(message).await.map_err(|e| e.to_string())
            }
            _ => Err(format!("Unsupported provider: {}", self.provider)),
        }
    }

    /// Chat with history (non-streaming)
    pub async fn chat(&self, message: &str, history: Vec<Message>) -> Result<String, String> {
        self.check_api_key()?;
        let preamble = self.preamble.clone().unwrap_or_default();

        match self.provider.as_str() {
            "openai" => {
                let client = if let Some(key) = &self.api_key {
                    openai::Client::new(key)
                } else {
                    Ok(openai::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent
                    .chat(message, history)
                    .await
                    .map(|r| r.to_string())
                    .map_err(|e| e.to_string())
            }
            "gemini" => {
                let client = if let Some(key) = &self.api_key {
                    gemini::Client::new(key)
                } else {
                    Ok(gemini::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent
                    .chat(message, history)
                    .await
                    .map(|r| r.to_string())
                    .map_err(|e| e.to_string())
            }
            "anthropic" => {
                let client = if let Some(key) = &self.api_key {
                    anthropic::Client::new(key)
                } else {
                    Ok(anthropic::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent
                    .chat(message, history)
                    .await
                    .map(|r| r.to_string())
                    .map_err(|e| e.to_string())
            }
            _ => Err(format!("Unsupported provider: {}", self.provider)),
        }
    }

    /// Streaming prompt - calls the callback for each token
    pub async fn stream_prompt<F>(&self, message: &str, on_token: &F) -> Result<String, String>
    where
        F: Fn(String) + Send + Sync,
    {
        self.check_api_key()?;
        let preamble = self.preamble.clone().unwrap_or_default();
        let mut full_response = String::new();

        match self.provider.as_str() {
            "openai" => {
                let client = if let Some(key) = &self.api_key {
                    openai::Client::new(key)
                } else {
                    Ok(openai::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_prompt(message).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(
                                StreamedAssistantContent::Text(t),
                            ) = token
                            {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        }
                        Err(e) => {
                            error!("Error in stream: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
            }
            "gemini" => {
                let client = if let Some(key) = &self.api_key {
                    gemini::Client::new(key)
                } else {
                    Ok(gemini::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_prompt(message).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(
                                StreamedAssistantContent::Text(t),
                            ) = token
                            {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        }
                        Err(e) => {
                            error!("Error in stream: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
            }
            "anthropic" => {
                let client = if let Some(key) = &self.api_key {
                    anthropic::Client::new(key)
                } else {
                    Ok(anthropic::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_prompt(message).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(
                                StreamedAssistantContent::Text(t),
                            ) = token
                            {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        }
                        Err(e) => {
                            error!("Error in stream: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
            }
            _ => return Err(format!("Unsupported provider: {}", self.provider)),
        }

        Ok(full_response)
    }

    /// Streaming chat with history - calls the callback for each token
    pub async fn stream_chat<F>(
        &self,
        message: &str,
        history: Vec<Message>,
        on_token: F,
    ) -> Result<String, String>
    where
        F: Fn(String) + Send + Sync,
    {
        self.check_api_key()?;
        let preamble = self.preamble.clone().unwrap_or_default();
        let mut full_response = String::new();

        match self.provider.as_str() {
            "openai" => {
                let client = if let Some(key) = &self.api_key {
                    openai::Client::new(key)
                } else {
                    Ok(openai::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_chat(message, history).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(
                                StreamedAssistantContent::Text(t),
                            ) = token
                            {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        }
                        Err(e) => {
                            error!("Error in stream: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
            }
            "gemini" => {
                let client = if let Some(key) = &self.api_key {
                    gemini::Client::new(key)
                } else {
                    Ok(gemini::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_chat(message, history).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(
                                StreamedAssistantContent::Text(t),
                            ) = token
                            {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        }
                        Err(e) => {
                            error!("Error in stream: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
            }
            "anthropic" => {
                let client = if let Some(key) = &self.api_key {
                    anthropic::Client::new(key)
                } else {
                    Ok(anthropic::Client::from_env())
                }.map_err(|e| e.to_string())?;
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_chat(message, history).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(
                                StreamedAssistantContent::Text(t),
                            ) = token
                            {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        }
                        Err(e) => {
                            error!("Error in stream: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
            }
            _ => return Err(format!("Unsupported provider: {}", self.provider)),
        }

        Ok(full_response)
    }

    /// Get a fast/cheap model for the current provider (useful for classification, titles, etc.)
    pub fn fast_model(provider: &str) -> &'static str {
        match provider {
            "gemini" => "gemini-2.0-flash",
            "anthropic" => "claude-3-haiku-20240307",
            "openai" => "gpt-4o-mini",
            _ => "gpt-4o-mini",
        }
    }
}

/// Helper to create Message for history
pub fn user_message(content: &str) -> Message {
    Message::user(content)
}

pub fn assistant_message(content: &str) -> Message {
    Message::assistant(content)
}
