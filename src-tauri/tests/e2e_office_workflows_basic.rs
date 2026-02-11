//! End-to-end tests for basic office workflow communication patterns
//!
//! Tests Phase 1: Foundation Tests
//! - Simple agent-to-agent messaging
//! - Group communication with multiple recipients
//! - Task assignment and progress tracking

mod office_test_helpers;

use anycowork::commands::mail::{get_mail_thread_messages, reply_to_mail, send_mail};
use anycowork::AppState;
use office_test_helpers::*;
use tauri::Manager;

/// Test 1.1: Simple Agent-to-Agent Message
///
/// Scenario: Alice sends email to Bob → Bob replies → Alice reads reply
/// Validates: Basic mail flow and thread continuity
#[tokio::test]
async fn test_simple_agent_to_agent_message() -> Result<(), Box<dyn std::error::Error>> {
    // Skip if no API key
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    // 1. Setup: Create test database and app state
    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // 2. Create agents with clear roles
    let alice = create_test_agent(
        pool,
        "Alice PM",
        "📋",
        "Project manager coordinating team tasks",
    );
    let bob = create_test_agent(
        pool,
        "Bob Developer",
        "💻",
        "Senior backend developer",
    );

    // 3. Alice sends email to Bob
    let thread = send_mail(
        state_handle.clone(),
        Some(alice.id.clone()),
        Some(bob.id.clone()),
        "Task Status Check".to_string(),
        "Hi Bob,\n\nCan you update me on the API endpoint progress?\n\nThanks,\nAlice"
            .to_string(),
    )
    .await?;

    assert_eq!(thread.subject, "Task Status Check");
    println!("✓ Alice sent email to Bob");

    // 4. Wait for background mail processing (Bob receives email)
    wait_for_mail_processing(state_handle.clone(), &bob.id, 1, 30).await?;
    println!("✓ Bob received email");

    // 5. Verify Bob received the email in inbox
    let bob_inbox = get_inbox(state_handle.clone(), &bob.id).await?;
    assert_eq!(bob_inbox.len(), 1, "Bob should have 1 email in inbox");
    assert_eq!(bob_inbox[0].subject, "Task Status Check");

    // 6. Verify Alice has the email in sent folder
    let alice_sent = get_sent_folder(state_handle.clone(), &alice.id).await?;
    assert_eq!(alice_sent.len(), 1, "Alice should have 1 email in sent folder");
    assert_eq!(alice_sent[0].subject, "Task Status Check");
    println!("✓ Email appears in correct folders");

    // 7. Bob replies to Alice
    let _reply = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(bob.id.clone()),
        "Hi Alice,\n\nThe API endpoint is 80% complete. Should be done by EOD.\n\nBest,\nBob"
            .to_string(),
    )
    .await?;

    println!("✓ Bob replied to Alice");

    // 8. Wait for Alice to receive reply
    wait_for_mail_processing(state_handle.clone(), &alice.id, 1, 30).await?;
    println!("✓ Alice received Bob's reply");

    // 9. Verify thread now has 2 messages
    let thread_messages = get_mail_thread_messages(state_handle.clone(), thread.id).await?;
    assert_eq!(
        thread_messages.len(),
        2,
        "Thread should have 2 messages (original + reply)"
    );
    assert!(thread_messages[0].content.contains("API endpoint progress"));
    assert!(thread_messages[1].content.contains("80% complete"));
    println!("✓ Thread maintains continuity");

    println!("\n✅ Test 1.1 PASSED: Simple agent-to-agent messaging works correctly");
    Ok(())
}

/// Test 1.2: Group Communication (3+ Agents)
///
/// Scenario: Alice broadcasts to Bob, Carol, Dave → All receive → Multiple replies converge
/// Validates: Multi-recipient handling
#[tokio::test]
async fn test_group_communication() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // Create agents
    let alice = create_test_agent(pool, "Alice PM", "📋", "Project manager");
    let bob = create_test_agent(pool, "Bob Dev", "💻", "Backend developer");
    let carol = create_test_agent(pool, "Carol Designer", "🎨", "UX designer");
    let dave = create_test_agent(pool, "Dave Tester", "🧪", "QA engineer");

    // Alice sends to Bob
    let thread1 = send_mail(
        state_handle.clone(),
        Some(alice.id.clone()),
        Some(bob.id.clone()),
        "Team Update: Sprint Planning".to_string(),
        "Hi Bob,\n\nPlease review the sprint plan for next week.\n\nAlice".to_string(),
    )
    .await?;

    // Alice sends to Carol
    let thread2 = send_mail(
        state_handle.clone(),
        Some(alice.id.clone()),
        Some(carol.id.clone()),
        "Team Update: Sprint Planning".to_string(),
        "Hi Carol,\n\nPlease review the sprint plan for next week.\n\nAlice".to_string(),
    )
    .await?;

    // Alice sends to Dave
    let thread3 = send_mail(
        state_handle.clone(),
        Some(alice.id.clone()),
        Some(dave.id.clone()),
        "Team Update: Sprint Planning".to_string(),
        "Hi Dave,\n\nPlease review the sprint plan for next week.\n\nAlice".to_string(),
    )
    .await?;

    println!("✓ Alice sent emails to 3 team members");

    // Wait for all to receive
    wait_for_mail_processing(state_handle.clone(), &bob.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &carol.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &dave.id, 1, 30).await?;
    println!("✓ All team members received emails");

    // Verify everyone received the email
    let bob_inbox = get_inbox(state_handle.clone(), &bob.id).await?;
    let carol_inbox = get_inbox(state_handle.clone(), &carol.id).await?;
    let dave_inbox = get_inbox(state_handle.clone(), &dave.id).await?;

    assert_eq!(bob_inbox.len(), 1, "Bob should have 1 email");
    assert_eq!(carol_inbox.len(), 1, "Carol should have 1 email");
    assert_eq!(dave_inbox.len(), 1, "Dave should have 1 email");
    println!("✓ All inboxes contain the message");

    // Bob and Carol reply
    let _bob_reply = reply_to_mail(
        state_handle.clone(),
        thread1.id.clone(),
        Some(bob.id.clone()),
        "Looks good to me! Ready to start.\n\nBob".to_string(),
    )
    .await?;

    let _carol_reply = reply_to_mail(
        state_handle.clone(),
        thread2.id.clone(),
        Some(carol.id.clone()),
        "I'll have the designs ready by Tuesday.\n\nCarol".to_string(),
    )
    .await?;

    println!("✓ Bob and Carol replied");

    // Wait for Alice to receive replies (she should have 2 emails in inbox from replies)
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    wait_for_mail_processing(state_handle.clone(), &alice.id, 2, 30).await?;
    println!("✓ Alice received 2 replies");

    // Verify Alice received 2 replies in her inbox
    let alice_inbox = get_inbox(state_handle.clone(), &alice.id).await?;
    assert!(
        alice_inbox.len() >= 2,
        "Alice should have at least 2 emails in inbox (replies from Bob and Carol)"
    );

    // Verify Alice sent folder has 3 emails (to Bob, Carol, Dave)
    let alice_sent = get_sent_folder(state_handle.clone(), &alice.id).await?;
    assert_eq!(
        alice_sent.len(),
        3,
        "Alice should have 3 emails in sent folder"
    );

    // Verify thread continuity
    let thread1_messages =
        get_mail_thread_messages(state_handle.clone(), thread1.id).await?;
    let thread2_messages =
        get_mail_thread_messages(state_handle.clone(), thread2.id).await?;
    let thread3_messages =
        get_mail_thread_messages(state_handle.clone(), thread3.id).await?;

    assert_eq!(thread1_messages.len(), 2, "Bob thread should have 2 messages");
    assert_eq!(
        thread2_messages.len(),
        2,
        "Carol thread should have 2 messages"
    );
    assert_eq!(
        thread3_messages.len(),
        1,
        "Dave thread should have 1 message (no reply yet)"
    );

    println!("\n✅ Test 1.2 PASSED: Group communication with multiple recipients works");
    Ok(())
}

/// Test 3.1: Task Assignment and Progress Updates
///
/// Scenario: PM assigns tasks to multiple agents → Tracks progress through email threads
/// Validates: Asynchronous coordination via email
#[tokio::test]
async fn test_task_assignment_and_progress() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // Create agents
    let pm = create_test_agent(
        pool,
        "Jordan PM",
        "📋",
        "Project manager responsible for task coordination",
    );
    let dev1 = create_test_agent(
        pool,
        "Alex Backend",
        "💻",
        "Backend developer specializing in APIs",
    );
    let dev2 = create_test_agent(
        pool,
        "Sam Frontend",
        "🖥️",
        "Frontend developer specializing in React",
    );

    // PM assigns task to Alex
    let task1 = send_mail(
        state_handle.clone(),
        Some(pm.id.clone()),
        Some(dev1.id.clone()),
        "Task: Implement User Authentication API".to_string(),
        "Hi Alex,\n\nPlease implement the user authentication API with JWT tokens. Priority: High.\n\nDeadline: End of week.\n\nJordan".to_string(),
    )
    .await?;

    // PM assigns task to Sam
    let task2 = send_mail(
        state_handle.clone(),
        Some(pm.id.clone()),
        Some(dev2.id.clone()),
        "Task: Build Login UI Component".to_string(),
        "Hi Sam,\n\nPlease build the login UI component with form validation.\n\nDeadline: End of week.\n\nJordan".to_string(),
    )
    .await?;

    println!("✓ PM assigned 2 tasks");

    // Wait for both to receive
    wait_for_mail_processing(state_handle.clone(), &dev1.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &dev2.id, 1, 30).await?;
    println!("✓ Both developers received tasks");

    // Alex sends progress update
    let _update1 = reply_to_mail(
        state_handle.clone(),
        task1.id.clone(),
        Some(dev1.id.clone()),
        "Hi Jordan,\n\nAPI implementation is 50% done. JWT generation is complete, working on validation now.\n\nAlex".to_string(),
    )
    .await?;

    // Sam sends progress update
    let _update2 = reply_to_mail(
        state_handle.clone(),
        task2.id.clone(),
        Some(dev2.id.clone()),
        "Hi Jordan,\n\nLogin UI mockup is ready. Starting implementation today.\n\nSam".to_string(),
    )
    .await?;

    println!("✓ Both developers sent progress updates");

    // Wait for PM to receive updates
    wait_for_mail_processing(state_handle.clone(), &pm.id, 2, 30).await?;
    println!("✓ PM received 2 progress updates");

    // Verify PM's inbox has 2 updates
    let pm_inbox = get_inbox(state_handle.clone(), &pm.id).await?;
    assert_eq!(
        pm_inbox.len(),
        2,
        "PM should have 2 emails in inbox (progress updates)"
    );

    // Verify PM's sent folder has 2 task assignments
    let pm_sent = get_sent_folder(state_handle.clone(), &pm.id).await?;
    assert_eq!(
        pm_sent.len(),
        2,
        "PM should have 2 emails in sent folder (task assignments)"
    );

    // Verify thread structure
    let task1_messages = get_mail_thread_messages(state_handle.clone(), task1.id).await?;
    let task2_messages = get_mail_thread_messages(state_handle.clone(), task2.id).await?;

    assert_eq!(
        task1_messages.len(),
        2,
        "Task 1 thread should have assignment + update"
    );
    assert_eq!(
        task2_messages.len(),
        2,
        "Task 2 thread should have assignment + update"
    );

    // Verify content
    assert!(task1_messages[0].content.contains("user authentication"));
    assert!(task1_messages[1].content.contains("50% done"));

    assert!(task2_messages[0].content.contains("login UI"));
    assert!(task2_messages[1].content.contains("mockup is ready"));

    println!("\n✅ Test 3.1 PASSED: Task assignment and progress tracking works");
    Ok(())
}
