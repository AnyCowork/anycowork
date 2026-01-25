pub mod agents;
pub mod commands;
pub mod database;
pub mod events;
pub mod mcp;
pub mod models;
pub mod permissions;
pub mod schema;
pub mod snapshots;
pub mod telegram;
pub mod tools;

use dashmap::DashMap;
use database::DbPool;
use permissions::PermissionManager;
use std::sync::Arc;
use telegram::TelegramBotManager;
use tokio::sync::oneshot;

// AppState definition
pub struct AppState {
    pub db_pool: DbPool,
    pub pending_approvals: Arc<DashMap<String, oneshot::Sender<bool>>>,
    pub telegram_manager: Arc<TelegramBotManager>,
    pub permission_manager: Arc<PermissionManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load env vars
    dotenvy::dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Setup DB
    let pool = database::establish_connection();
    database::run_migrations(&pool);

    // Create default agent if none exist
    database::ensure_default_agent(&pool);

    let pending_approvals = Arc::new(DashMap::new());
    let telegram_manager = Arc::new(TelegramBotManager::new(pool.clone()));
    let permission_manager = Arc::new(PermissionManager::new());

    // Clone for async startup task
    let telegram_manager_clone = telegram_manager.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db_pool: pool,
            pending_approvals,
            telegram_manager,
            permission_manager,
        })
        .setup(move |_app| {
            // Start all active Telegram bots on app startup
            let manager = telegram_manager_clone.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = manager.start_all_active_bots().await {
                    log::error!("Failed to start active Telegram bots: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_agent,
            commands::get_agents,
            commands::update_agent,
            commands::chat,
            commands::approve_action,
            commands::reject_action,
            commands::create_session,
            commands::get_sessions,
            commands::update_session,
            commands::delete_session,
            commands::get_session_messages,
            commands::get_session_with_messages,
            commands::add_message,
            commands::delete_message,
            commands::get_session_stats,
            commands::create_telegram_config,
            commands::get_telegram_configs,
            commands::get_telegram_config,
            commands::update_telegram_config,
            commands::delete_telegram_config,
            commands::start_telegram_bot,
            commands::stop_telegram_bot,
            commands::get_telegram_bot_status,
            commands::get_running_telegram_bots,
            // Page commands
            commands::create_page,
            commands::get_pages,
            commands::get_page,
            commands::update_page,
            commands::archive_page,
            commands::restore_page,
            commands::delete_page,
            // Block commands
            commands::get_page_blocks,
            commands::create_block,
            commands::update_block,
            commands::delete_block,
            commands::batch_update_blocks,
            // Attachment commands
            commands::upload_attachment,
            commands::get_page_attachments,
            commands::delete_attachment,
            // Skill commands
            commands::get_skills,
            commands::get_skill,
            commands::create_skill,
            commands::update_skill,
            commands::delete_skill,
            commands::toggle_skill,
            // Window commands
            commands::toggle_devtools,
            commands::is_dev_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
