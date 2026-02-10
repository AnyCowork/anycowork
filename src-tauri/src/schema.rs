diesel::table! {
    tasks (id) {
        id -> Text,
        title -> Text,
        description -> Nullable<Text>,
        status -> Text,
        priority -> Integer,
        session_id -> Nullable<Text>,
        agent_id -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}
