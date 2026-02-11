diesel::table! {
    agents (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        status -> Text,
        personality -> Nullable<Text>,
        tone -> Nullable<Text>,
        expertise -> Nullable<Text>,
        ai_provider -> Text,
        ai_model -> Text,
        ai_temperature -> Float,
        ai_config -> Text,
        system_prompt -> Nullable<Text>,
        permissions -> Nullable<Text>,
        working_directories -> Nullable<Text>,
        skills -> Nullable<Text>,
        mcp_servers -> Nullable<Text>,
        messaging_connections -> Nullable<Text>,
        knowledge_bases -> Nullable<Text>,
        api_keys -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
        platform_configs -> Nullable<Text>,
        execution_settings -> Nullable<Text>,
        scope_type -> Nullable<Text>,
        workspace_path -> Nullable<Text>,
        avatar -> Nullable<Text>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        agent_id -> Text,
        title -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        archived -> Integer,
        pinned -> Integer,
    }
}

diesel::table! {
    messages (id) {
        id -> Text,
        role -> Text,
        content -> Text,
        session_id -> Text,
        created_at -> Timestamp,
        metadata_json -> Nullable<Text>,
        tokens -> Nullable<Integer>,
    }
}

diesel::table! {
    telegram_configs (id) {
        id -> Text,
        bot_token -> Text,
        agent_id -> Text,
        is_active -> Integer,
        allowed_chat_ids -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    pages (id) {
        id -> Text,
        title -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        parent_id -> Nullable<Text>,
        day_date -> Nullable<Text>,
        icon -> Nullable<Text>,
        cover_image -> Nullable<Text>,
        is_archived -> Integer,
        is_published -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    blocks (id) {
        id -> Text,
        page_id -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        content_json -> Text,
        order_index -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    attachments (id) {
        id -> Text,
        page_id -> Text,
        block_id -> Nullable<Text>,
        file_path -> Text,
        file_name -> Text,
        file_type -> Text,
        file_size -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    agent_skills (id) {
        id -> Text,
        name -> Text,
        display_title -> Text,
        description -> Text,
        skill_content -> Text,
        additional_files_json -> Nullable<Text>,
        enabled -> Integer,
        version -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        source_path -> Nullable<Text>,
        category -> Nullable<Text>,
        requires_sandbox -> Integer,
        sandbox_config -> Nullable<Text>,
        execution_mode -> Text,
    }
}

diesel::table! {
    skill_files (id) {
        id -> Text,
        skill_id -> Text,
        relative_path -> Text,
        content -> Text,
        file_type -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    agent_skill_assignments (agent_id, skill_id) {
        agent_id -> Text,
        skill_id -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    mail_threads (id) {
        id -> Text,
        subject -> Text,
        is_read -> Integer,
        is_archived -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    mail_messages (id) {
        id -> Text,
        thread_id -> Text,
        sender_type -> Text,
        sender_agent_id -> Nullable<Text>,
        recipient_type -> Text,
        recipient_agent_id -> Nullable<Text>,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    settings (id) {
        id -> Text,
        key -> Text,
        value -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(messages -> sessions (session_id));
diesel::joinable!(sessions -> agents (agent_id));
diesel::joinable!(telegram_configs -> agents (agent_id));
diesel::joinable!(blocks -> pages (page_id));
diesel::joinable!(attachments -> pages (page_id));
diesel::joinable!(agent_skill_assignments -> agents (agent_id));
diesel::joinable!(agent_skill_assignments -> agent_skills (skill_id));
diesel::joinable!(skill_files -> agent_skills (skill_id));
diesel::joinable!(mail_messages -> mail_threads (thread_id));

diesel::allow_tables_to_appear_in_same_query!(
    agents,
    messages,
    sessions,
    telegram_configs,
    pages,
    blocks,
    attachments,
    agent_skills,
    agent_skill_assignments,
    skill_files,
    mcp_servers,
    mail_threads,
    mail_messages,
    settings,
);

diesel::table! {
    mcp_servers (id) {
        id -> Text,
        name -> Text,
        server_type -> Text,
        command -> Nullable<Text>,
        args -> Nullable<Text>,
        env -> Nullable<Text>,
        url -> Nullable<Text>,
        is_enabled -> Integer,
        template_id -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}
