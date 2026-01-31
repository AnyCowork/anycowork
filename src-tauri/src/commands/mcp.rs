use crate::models::mcp_server::{McpServer, McpServerDto, McpServerUpdateDto, McpTemplateDto, NewMcpServer};
use crate::models::{Agent, AgentDto};
use crate::schema;
use crate::AppState;
use diesel::prelude::*;
use tauri::State;

#[tauri::command]
pub async fn get_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServerDto>, String> {
    use schema::mcp_servers::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let results = mcp_servers
        .load::<McpServer>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|s| s.into_dto()).collect())
}

#[tauri::command]
pub async fn create_mcp_server(
    state: State<'_, AppState>,
    data: McpServerUpdateDto,
    template_id: Option<String>,
) -> Result<McpServerDto, String> {
    use schema::mcp_servers;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let new_id = uuid::Uuid::new_v4().to_string();
    let name = data.name.unwrap_or_else(|| "New MCP Server".to_string());
    let server_type = data.server_type.unwrap_or_else(|| "stdio".to_string());
    
    let args_json = data.args.map(|a| serde_json::to_string(&a).unwrap_or_default());
    let env_json = data.env.map(|e| serde_json::to_string(&e).unwrap_or_default());

    let new_server = NewMcpServer {
        id: new_id.clone(),
        name,
        server_type,
        command: data.command,
        args: args_json,
        env: env_json,
        url: data.url,
        is_enabled: if data.is_enabled.unwrap_or(true) { 1 } else { 0 },
        template_id,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    diesel::insert_into(mcp_servers::table)
        .values(&new_server)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let server: McpServer = mcp_servers::table
        .filter(mcp_servers::id.eq(new_id))
        .first::<McpServer>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(server.into_dto())
}

#[tauri::command]
pub async fn update_mcp_server(
    state: State<'_, AppState>,
    id: String,
    data: McpServerUpdateDto,
) -> Result<McpServerDto, String> {
    use schema::mcp_servers::dsl::{mcp_servers, id as id_col, name, server_type, command, args, env, url, is_enabled, updated_at};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let mut server = mcp_servers
        .filter(id_col.eq(&id))
        .first::<McpServer>(&mut conn)
        .map_err(|_| "MCP Server not found".to_string())?;

    if let Some(n) = data.name {
        server.name = n;
    }
    if let Some(t) = data.server_type {
        server.server_type = t;
    }
    if let Some(c) = data.command {
        server.command = Some(c);
    }
    if let Some(a) = data.args {
        server.args = Some(serde_json::to_string(&a).unwrap_or_default());
    }
    if let Some(e) = data.env {
        server.env = Some(serde_json::to_string(&e).unwrap_or_default());
    }
    if let Some(u) = data.url {
        server.url = Some(u);
    }
    if let Some(e) = data.is_enabled {
        server.is_enabled = if e { 1 } else { 0 };
    }

    server.updated_at = chrono::Utc::now().timestamp();

    diesel::update(mcp_servers.filter(id_col.eq(&id)))
        .set((
            name.eq(&server.name),
            server_type.eq(&server.server_type),
            command.eq(&server.command),
            args.eq(&server.args),
            env.eq(&server.env),
            url.eq(&server.url),
            is_enabled.eq(&server.is_enabled),
            updated_at.eq(&server.updated_at),
        ))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(server.into_dto())
}

#[tauri::command]
pub async fn delete_mcp_server(state: State<'_, AppState>, id: String) -> Result<(), String> {
    use schema::mcp_servers::dsl::{mcp_servers, id as id_col};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    diesel::delete(mcp_servers.filter(id_col.eq(&id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_mcp_templates() -> Result<Vec<McpTemplateDto>, String> {
    let templates = vec![
        McpTemplateDto {
            id: "brave-search".to_string(),
            name: "Brave Search".to_string(),
            description: "Search the web using Brave Search API".to_string(),
            server_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()]),
            env: Some(serde_json::json!({ "BRAVE_API_KEY": "YOUR_API_KEY" })),
            url: None,
        },
        McpTemplateDto {
            id: "filesystem".to_string(),
            name: "Filesystem".to_string(),
            description: "Access local filesystem".to_string(),
            server_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string(), "/path/to/allow".to_string()]),
            env: None,
            url: None,
        },
        McpTemplateDto {
            id: "memory".to_string(),
            name: "Memory".to_string(),
            description: "Persistent memory for the agent".to_string(),
            server_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()]),
            env: None,
            url: None,
        },
        McpTemplateDto {
            id: "fetch".to_string(),
            name: "Fetch".to_string(),
            description: "Fetch content from URLs".to_string(),
            server_type: "stdio".to_string(),
            command: Some("uvx".to_string()),
            args: Some(vec!["mcp-server-fetch".to_string()]),
            env: None,
            url: None,
        },
    ];

    Ok(templates)
}

#[tauri::command]
pub async fn add_mcp_to_agent(
    state: State<'_, AppState>,
    agent_id: String,
    mcp_server_id: String,
) -> Result<AgentDto, String> {
    use schema::agents::dsl::{agents, id as agent_id_col, mcp_servers as mcp_servers_col};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Verify MCP server exists
    use schema::mcp_servers::dsl::{mcp_servers as mcp_servers_dsl, id as mcp_id_col};
    let _ = mcp_servers_dsl
        .filter(mcp_id_col.eq(&mcp_server_id))
        .first::<McpServer>(&mut conn)
        .map_err(|_| "MCP Server not found".to_string())?;

    // Get Agent
    let mut agent = agents
        .filter(agent_id_col.eq(&agent_id))
        .first::<Agent>(&mut conn)
        .map_err(|_| "Agent not found".to_string())?;

    let mut current_servers: Vec<String> = agent
        .mcp_servers
        .clone()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    if !current_servers.contains(&mcp_server_id) {
        current_servers.push(mcp_server_id);
    }

    let new_servers_str = current_servers.join(", ");
    agent.mcp_servers = Some(new_servers_str.clone());

    diesel::update(agents.filter(agent_id_col.eq(&agent_id)))
        .set(mcp_servers_col.eq(new_servers_str))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(agent.into_dto())
}

#[tauri::command]
pub async fn remove_mcp_from_agent(
    state: State<'_, AppState>,
    agent_id: String,
    mcp_server_id: String,
) -> Result<AgentDto, String> {
    use schema::agents::dsl::{agents, id as agent_id_col, mcp_servers as mcp_servers_col};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get Agent
    let mut agent = agents
        .filter(agent_id_col.eq(&agent_id))
        .first::<Agent>(&mut conn)
        .map_err(|_| "Agent not found".to_string())?;

    let mut current_servers: Vec<String> = agent
        .mcp_servers
        .clone()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    if let Some(pos) = current_servers.iter().position(|x| *x == mcp_server_id) {
        current_servers.remove(pos);
    }

    let new_servers_str = current_servers.join(", ");
    agent.mcp_servers = Some(new_servers_str.clone());

    diesel::update(agents.filter(agent_id_col.eq(&agent_id)))
        .set(mcp_servers_col.eq(new_servers_str))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(agent.into_dto())
}
