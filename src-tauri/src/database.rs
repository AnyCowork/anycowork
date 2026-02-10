use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn run_migrations(pool: &anyagents::database::DbPool) {
    log::info!("Checking for pending database migrations...");
    let mut conn = pool.get().expect("Failed to get connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    log::info!("Database migrations executed successfully.");
}
