use crate::models::NewAgent;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_connection() -> DbPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME environment variable not set");
        let path = std::path::Path::new(&home).join(".anycowork");
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Failed to create .anycowork directory");
        }
        let db_path = path.join("anycowork.db");
        format!("sqlite://{}", db_path.to_string_lossy())
    });

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn run_migrations(pool: &DbPool) {
    log::info!("Checking for pending database migrations...");
    let mut conn = pool.get().expect("Failed to get connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    log::info!("Database migrations executed successfully.");
}

pub fn ensure_default_agent(pool: &DbPool) {
    use crate::schema::agents::dsl::*;

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to get connection for default agent check: {}", e);
            return;
        }
    };

    // Check if any agents exist
    let count: i64 = match agents.count().get_result(&mut conn) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to count agents: {}", e);
            return;
        }
    };

    if count == 0 {
        log::info!("No agents found, creating default agent...");

        let default_agent = NewAgent {
            id: uuid::Uuid::new_v4().to_string(),
            name: "AnyCoworker Default".to_string(),
            description: Some("Default AI assistant for general tasks".to_string()),
            status: "active".to_string(),
            personality: None,
            tone: None,
            expertise: None,
            ai_provider: "gemini".to_string(),
            ai_model: "gemini-3-flash-preview".to_string(),
            ai_temperature: 0.7,
            ai_config: r#"{"provider": "gemini", "model": "gemini-3-flash-preview"}"#.to_string(),
            system_prompt: Some(
                r#"You are an intelligent AI Coworker designed to help with daily office tasks.
Your goal is to be proactive, organized, and helpful.
You should:
1. Ask clarifying questions when requirements are vague.
2. Build and maintain a todo list for complex tasks.
3. Use available tools to search for information, manage files, and execute code.
4. Report progress regularly."#
                    .to_string(),
            ),
            permissions: None,
            working_directories: None,
            skills: None,
            mcp_servers: None,
            messaging_connections: None,
            knowledge_bases: None,
            api_keys: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            platform_configs: None,
            execution_settings: None,
            scope_type: None,
            workspace_path: None,
        };

        match diesel::insert_into(agents)
            .values(&default_agent)
            .execute(&mut conn)
        {
            Ok(_) => log::info!("Default agent created successfully"),
            Err(e) => log::error!("Failed to create default agent: {}", e),
        }
    }
}

// Helper to setup an in-memory database for testing
pub fn create_test_pool() -> DbPool {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("anycowork_test_{}.db", uuid::Uuid::new_v4()));
    let db_str = db_path.to_string_lossy().to_string();

    let manager = ConnectionManager::<SqliteConnection>::new(db_str);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create test pool.");

    run_migrations(&pool);
    pool
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::agents::dsl::*;
    // use diesel::prelude::*; // Unused

    #[test]
    fn test_establish_connection() {
        let pool = create_test_pool();
        let conn = pool.get();
        assert!(conn.is_ok());
    }

    #[test]
    fn test_ensure_default_agent() {
        let pool = create_test_pool();

        // Initially no agents (clear any seeded by migrations)
        {
            let mut conn = pool.get().unwrap();
            diesel::delete(agents).execute(&mut conn).unwrap();
            let count: i64 = agents.count().get_result(&mut conn).unwrap();
            assert_eq!(count, 0);
        }

        // Run ensure_default_agent
        ensure_default_agent(&pool);

        // Should have 1 agent
        {
            let mut conn = pool.get().unwrap();
            let count: i64 = agents.count().get_result(&mut conn).unwrap();
            assert_eq!(count, 1);

            let agent = agents.first::<crate::models::Agent>(&mut conn).unwrap();
            assert_eq!(agent.name, "AnyCoworker Default");
        }

        // Run again, should still have 1 agent (idempotent)
        ensure_default_agent(&pool);

        {
            let mut conn = pool.get().unwrap();
            let count: i64 = agents.count().get_result(&mut conn).unwrap();
            assert_eq!(count, 1);
        }
    }
}
