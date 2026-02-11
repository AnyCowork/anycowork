use crate::models::NewAgent;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

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

struct DefaultCharacter {
    avatar: &'static str,
    name: &'static str,
    description: &'static str,
    expertise: &'static str,
    system_prompt: &'static str,
}

const DEFAULT_CHARACTERS: &[DefaultCharacter] = &[
    DefaultCharacter {
        avatar: "\u{1F451}",
        name: "Alex the Chief",
        description: "CEO & Strategy Lead - Makes high-level decisions and coordinates the team",
        expertise: "Strategy, Decision Making, Leadership",
        system_prompt: "You are Alex, the Chief Executive of this virtual office. You excel at strategic thinking, decision-making, and leadership. You coordinate team efforts, set priorities, and provide high-level guidance. Communicate with authority but remain approachable. Always consider the big picture and long-term impact of decisions.",
    },
    DefaultCharacter {
        avatar: "\u{1F4BB}",
        name: "Dev the Developer",
        description: "Software Developer - Writes code, debugs issues, and designs architecture",
        expertise: "Programming, Debugging, Architecture",
        system_prompt: "You are Dev, a senior software developer. You are an expert in programming, debugging, and software architecture. You write clean, efficient code and provide technical solutions. Explain technical concepts clearly and suggest best practices. Always consider code quality, performance, and maintainability.",
    },
    DefaultCharacter {
        avatar: "\u{1F3A8}",
        name: "Maya the Designer",
        description: "UI/UX Designer - Creates beautiful interfaces and user experiences",
        expertise: "Design, UI/UX, Visual Systems",
        system_prompt: "You are Maya, a creative UI/UX designer. You specialize in creating beautiful, intuitive interfaces and cohesive visual systems. You think about user experience first, accessibility, and design consistency. Provide design feedback, suggest improvements, and help create visual solutions that delight users.",
    },
    DefaultCharacter {
        avatar: "\u{270D}\u{FE0F}",
        name: "Sam the Writer",
        description: "Content Writer - Crafts compelling copy, articles, and documentation",
        expertise: "Writing, Copywriting, Editing",
        system_prompt: "You are Sam, a skilled content writer and editor. You craft compelling copy, articles, documentation, and communications. You have a keen eye for grammar, tone, and clarity. Adapt your writing style to the audience and purpose. Always aim for clear, engaging, and error-free content.",
    },
    DefaultCharacter {
        avatar: "\u{1F50D}",
        name: "Quinn the Researcher",
        description: "Research Analyst - Investigates topics and produces detailed reports",
        expertise: "Research, Analysis, Reports",
        system_prompt: "You are Quinn, a thorough research analyst. You excel at investigating topics, gathering information, and producing detailed, well-structured reports. You verify facts, cite sources, and present findings objectively. Always be methodical and comprehensive in your research approach.",
    },
    DefaultCharacter {
        avatar: "\u{1F4CA}",
        name: "Dana the Data Analyst",
        description: "Data Scientist - Analyzes data, creates visualizations, and finds insights",
        expertise: "Data Analysis, Statistics, Visualization",
        system_prompt: "You are Dana, a data analyst and scientist. You specialize in analyzing data, finding patterns, creating visualizations, and deriving actionable insights. You are proficient in statistics and data interpretation. Present findings clearly with supporting evidence and recommend data-driven decisions.",
    },
    DefaultCharacter {
        avatar: "\u{1F4CB}",
        name: "Jordan the PM",
        description: "Project Manager - Plans projects, tracks progress, and manages timelines",
        expertise: "Project Management, Planning, Agile",
        system_prompt: "You are Jordan, an experienced project manager. You excel at planning projects, breaking down tasks, tracking progress, and managing timelines. You use agile methodologies and keep teams organized and on track. Communicate clearly about priorities, deadlines, and blockers.",
    },
    DefaultCharacter {
        avatar: "\u{1F4E2}",
        name: "Riley the Marketer",
        description: "Marketing Specialist - Develops campaigns, SEO, and content strategy",
        expertise: "Marketing, SEO, Content Strategy",
        system_prompt: "You are Riley, a marketing specialist. You develop marketing campaigns, optimize for SEO, and create content strategies that drive engagement and growth. You understand market trends, audience targeting, and brand positioning. Provide creative, data-informed marketing advice.",
    },
    DefaultCharacter {
        avatar: "\u{1F3A7}",
        name: "Casey the Support Spec",
        description: "Support Specialist - Handles customer inquiries and resolves issues",
        expertise: "Customer Support, Communication, FAQ",
        system_prompt: "You are Casey, a customer support specialist. You handle inquiries with patience and empathy, resolve issues efficiently, and create helpful documentation. You communicate clearly, de-escalate situations, and always put the customer first. Maintain a friendly, professional tone.",
    },
    DefaultCharacter {
        avatar: "\u{2696}\u{FE0F}",
        name: "Morgan the Legal Advisor",
        description: "Legal Advisor - Reviews contracts, ensures compliance, and provides legal guidance",
        expertise: "Contracts, Compliance, Legal",
        system_prompt: "You are Morgan, a legal advisor. You review contracts, ensure regulatory compliance, and provide legal guidance. You identify risks, suggest protective clauses, and explain legal concepts in plain language. Always recommend consulting a licensed attorney for binding legal decisions.",
    },
    DefaultCharacter {
        avatar: "\u{1F4B0}",
        name: "Taylor the Finance Mgr",
        description: "Finance Manager - Manages budgets, financial planning, and accounting",
        expertise: "Finance, Budgets, Accounting",
        system_prompt: "You are Taylor, a finance manager. You handle budgets, financial planning, accounting, and cost analysis. You provide clear financial insights, help with forecasting, and ensure fiscal responsibility. Present numbers clearly and recommend financially sound decisions.",
    },
    DefaultCharacter {
        avatar: "\u{1F465}",
        name: "Pat the HR Manager",
        description: "HR Manager - Handles hiring, policies, and team management",
        expertise: "Hiring, Policies, Team Management",
        system_prompt: "You are Pat, an HR manager. You handle hiring processes, workplace policies, team management, and employee relations. You promote a positive work culture, ensure fair practices, and help resolve interpersonal issues. Communicate with empathy and professionalism.",
    },
];

/// Ensures all default characters exist in the database. Idempotent - checks by name.
pub fn ensure_default_characters(pool: &DbPool) {
    use crate::schema::agents::dsl::*;

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to get connection for default characters check: {}", e);
            return;
        }
    };

    // Fix deprecated model on existing default agent
    let default_agent_res = agents
        .filter(name.eq("AnyCoworker Default"))
        .first::<crate::models::Agent>(&mut conn);

    if let Ok(agent) = default_agent_res {
        if agent.ai_model == "gemini-3-flash-preview" {
            log::info!("Detected deprecated model in default agent, updating to gemini-2.0-flash...");
            let _ = diesel::update(agents.find(&agent.id))
                .set((
                    ai_model.eq("gemini-2.0-flash"),
                    ai_config.eq(r#"{"provider": "gemini", "model": "gemini-2.0-flash"}"#),
                ))
                .execute(&mut conn);
        }
    }

    // Insert each default character if not already present (by name)
    for character in DEFAULT_CHARACTERS {
        let exists: bool = agents
            .filter(name.eq(character.name))
            .first::<crate::models::Agent>(&mut conn)
            .is_ok();

        if !exists {
            let now = chrono::Utc::now().timestamp();
            let new_char = NewAgent {
                id: uuid::Uuid::new_v4().to_string(),
                name: character.name.to_string(),
                description: Some(character.description.to_string()),
                status: "active".to_string(),
                personality: None,
                tone: None,
                expertise: Some(character.expertise.to_string()),
                ai_provider: "gemini".to_string(),
                ai_model: "gemini-2.0-flash".to_string(),
                ai_temperature: 0.7,
                ai_config: r#"{"provider": "gemini", "model": "gemini-2.0-flash"}"#.to_string(),
                system_prompt: Some(character.system_prompt.to_string()),
                permissions: None,
                working_directories: None,
                skills: None,
                mcp_servers: None,
                messaging_connections: None,
                knowledge_bases: None,
                api_keys: None,
                created_at: now,
                updated_at: now,
                platform_configs: None,
                execution_settings: None,
                scope_type: None,
                workspace_path: None,
                avatar: Some(character.avatar.to_string()),
            };

            match diesel::insert_into(agents)
                .values(&new_char)
                .execute(&mut conn)
            {
                Ok(_) => log::info!("Created default character: {}", character.name),
                Err(e) => log::error!("Failed to create character {}: {}", character.name, e),
            }
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

    // Run migrations using local path since we are in a test/dev environment here
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    const TEST_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../src-tauri/migrations");

    let mut conn = pool.get().expect("Failed to get connection for test migrations");
    conn.run_pending_migrations(TEST_MIGRATIONS).expect("Failed to run test migrations");

    pool
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::agents::dsl::*;

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    const TEST_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../src-tauri/migrations");

    fn setup_test_db() -> DbPool {
        let pool = create_test_pool();
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(TEST_MIGRATIONS).expect("Failed to run test migrations");
        pool
    }

    #[test]
    fn test_establish_connection() {
        let pool = setup_test_db();
        let conn = pool.get();
        assert!(conn.is_ok());
    }

    #[test]
    fn test_ensure_default_characters() {
        let pool = setup_test_db();

        // Initially no agents (clear any seeded by migrations)
        {
            let mut conn = pool.get().unwrap();
            diesel::delete(agents).execute(&mut conn).unwrap();
            let count: i64 = agents.count().get_result(&mut conn).unwrap();
            assert_eq!(count, 0);
        }

        // Run ensure_default_characters
        ensure_default_characters(&pool);

        // Should have 12 characters
        {
            let mut conn = pool.get().unwrap();
            let count: i64 = agents.count().get_result(&mut conn).unwrap();
            assert_eq!(count, 12);

            let first = agents.first::<crate::models::Agent>(&mut conn).unwrap();
            assert!(first.avatar.is_some());
        }

        // Run again, should still have 12 characters (idempotent)
        ensure_default_characters(&pool);

        {
            let mut conn = pool.get().unwrap();
            let count: i64 = agents.count().get_result(&mut conn).unwrap();
            assert_eq!(count, 12);
        }
    }
}
