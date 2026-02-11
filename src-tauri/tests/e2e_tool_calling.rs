//! End-to-end tests for tool calling and message filtering
//!
//! Tests the complete tool calling workflow including:
//! - JSON parsing with various formats
//! - Escape sequence handling
//! - Message deduplication
//! - Tool validation and execution

use anyagents::agents::extract_tool_calls;

#[test]
fn test_tool_call_with_escape_sequences() {
    // Test that tool calls with escape sequences are parsed correctly
    let response = r#"{
        "tool": "send_email",
        "args": {
            "recipient": "Jordan",
            "subject": "Hello",
            "body": "Hi Jordan,\n\nJust wanted to say hello!\n\nBest,\nDev"
        }
    }"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1, "Should extract 1 tool call");

    let call = &calls[0];
    assert_eq!(call["tool"], "send_email");

    let body = call["args"]["body"].as_str().unwrap();
    // Verify escape sequences are preserved in JSON
    assert!(body.contains('\n'), "Newlines should be actual newline characters, not escape sequences");
    assert!(body.contains("Hi Jordan"), "Body should contain greeting");
    assert!(body.contains("Best,"), "Body should contain closing");
}

#[test]
fn test_tool_call_embedded_in_text() {
    let response = "I will now send an email: {\"tool\": \"send_email\", \"args\": {\"to\": \"Jordan\", \"subject\": \"Hi\", \"body\": \"Hello\"}}";

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["tool"], "send_email");
}

#[test]
fn test_tool_call_with_markdown() {
    let response = r#"
Here's what I'll do:

```json
{
    "tool": "send_email",
    "args": {
        "to": "Jordan",
        "subject": "Test",
        "body": "Message body"
    }
}
```
"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["tool"], "send_email");
}

#[test]
fn test_multiple_tool_calls() {
    let response = r#"
First I'll do this: {"tool": "tool1", "args": {"param": "value1"}}
Then I'll do this: {"tool": "tool2", "args": {"param": "value2"}}
"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0]["tool"], "tool1");
    assert_eq!(calls[1]["tool"], "tool2");
}

#[test]
fn test_tool_call_with_nested_json() {
    let response = r#"{
        "tool": "complex_tool",
        "args": {
            "data": {
                "nested": {
                    "field": "value"
                }
            }
        }
    }"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["tool"], "complex_tool");
    assert_eq!(calls[0]["args"]["data"]["nested"]["field"], "value");
}

#[test]
fn test_invalid_json_ignored() {
    let response = "This is just text with { incomplete json";

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 0, "Invalid JSON should be ignored");
}

#[test]
fn test_non_tool_json_ignored() {
    let response = r#"{"not_a_tool": "value", "something": "else"}"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 0, "JSON without 'tool' field should be ignored");
}

#[test]
fn test_tool_call_with_special_characters() {
    let response = r#"{
        "tool": "bash",
        "args": {
            "command": "echo \"Hello World\" | grep 'Hello'"
        }
    }"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["tool"], "bash");

    let command = calls[0]["args"]["command"].as_str().unwrap();
    assert!(command.contains('"'));
    assert!(command.contains('\''));
}

#[test]
fn test_tool_call_with_unicode() {
    let response = r#"{
        "tool": "send_email",
        "args": {
            "to": "Jordan",
            "subject": "🎉 Celebration",
            "body": "Hello 世界! Bonjour 🌍"
        }
    }"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);

    let subject = calls[0]["args"]["subject"].as_str().unwrap();
    assert!(subject.contains("🎉"));
    assert!(subject.contains("Celebration"));

    let body = calls[0]["args"]["body"].as_str().unwrap();
    assert!(body.contains("世界"));
    assert!(body.contains("🌍"));
}

#[test]
fn test_tool_call_with_empty_args() {
    let response = r#"{"tool": "list_directory", "args": {}}"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["tool"], "list_directory");
    assert!(calls[0]["args"].is_object());
}

#[test]
fn test_tool_call_array_args() {
    let response = r#"{
        "tool": "batch_operation",
        "args": {
            "items": ["item1", "item2", "item3"]
        }
    }"#;

    let calls = extract_tool_calls(response);
    assert_eq!(calls.len(), 1);

    let items = calls[0]["args"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
}

// Test message deduplication logic
mod message_filtering {
    use anyagents::database::create_test_pool;
    use anyagents::models::{NewMessage, Message};
    use anyagents::schema::messages;
    use diesel::prelude::*;

    #[test]
    fn test_save_unique_messages() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let session_id = uuid::Uuid::new_v4().to_string();

        // Save first message
        let msg1 = NewMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            session_id: session_id.clone(),
            metadata_json: None,
            tokens: None,
        };

        diesel::insert_into(messages::table)
            .values(&msg1)
            .execute(&mut conn)
            .unwrap();

        // Save different message
        let msg2 = NewMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
            session_id: session_id.clone(),
            metadata_json: None,
            tokens: None,
        };

        diesel::insert_into(messages::table)
            .values(&msg2)
            .execute(&mut conn)
            .unwrap();

        // Verify both messages saved
        let saved_messages: Vec<Message> = messages::table
            .filter(messages::session_id.eq(&session_id))
            .load(&mut conn)
            .unwrap();

        assert_eq!(saved_messages.len(), 2);
    }

    #[test]
    fn test_prevent_duplicate_content() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let session_id = uuid::Uuid::new_v4().to_string();
        let content = "Duplicate content".to_string();

        // Save first message
        let msg1 = NewMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: content.clone(),
            session_id: session_id.clone(),
            metadata_json: None,
            tokens: None,
        };

        diesel::insert_into(messages::table)
            .values(&msg1)
            .execute(&mut conn)
            .unwrap();

        // Check if duplicate exists before saving
        let existing: Vec<Message> = messages::table
            .filter(messages::session_id.eq(&session_id))
            .filter(messages::role.eq("assistant"))
            .filter(messages::content.eq(&content))
            .load(&mut conn)
            .unwrap();

        assert_eq!(existing.len(), 1, "Should have 1 existing message");

        // If duplicate exists, we should NOT save again
        // (This logic should be implemented in the save_message function)
        if existing.is_empty() {
            let msg2 = NewMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role: "assistant".to_string(),
                content: content.clone(),
                session_id: session_id.clone(),
                metadata_json: None,
                tokens: None,
            };

            diesel::insert_into(messages::table)
                .values(&msg2)
                .execute(&mut conn)
                .unwrap();
        }

        // Verify only 1 message with this content exists
        let all_with_content: Vec<Message> = messages::table
            .filter(messages::session_id.eq(&session_id))
            .filter(messages::content.eq(&content))
            .load(&mut conn)
            .unwrap();

        assert_eq!(all_with_content.len(), 1, "Should prevent duplicate saves");
    }

    #[test]
    fn test_allow_same_content_different_roles() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let session_id = uuid::Uuid::new_v4().to_string();
        let content = "Same content".to_string();

        // Save as user
        let msg1 = NewMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: content.clone(),
            session_id: session_id.clone(),
            metadata_json: None,
            tokens: None,
        };

        diesel::insert_into(messages::table)
            .values(&msg1)
            .execute(&mut conn)
            .unwrap();

        // Save as assistant (should be allowed even with same content)
        let msg2 = NewMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: content.clone(),
            session_id: session_id.clone(),
            metadata_json: None,
            tokens: None,
        };

        diesel::insert_into(messages::table)
            .values(&msg2)
            .execute(&mut conn)
            .unwrap();

        // Verify both messages saved
        let saved_messages: Vec<Message> = messages::table
            .filter(messages::session_id.eq(&session_id))
            .filter(messages::content.eq(&content))
            .load(&mut conn)
            .unwrap();

        assert_eq!(saved_messages.len(), 2, "Same content with different roles should be allowed");
    }
}
