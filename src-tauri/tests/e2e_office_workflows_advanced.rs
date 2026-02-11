//! End-to-end tests for advanced workflow patterns and edge cases
//!
//! Tests Phase 4: Advanced Tests
//! - Email forwarding and CC patterns
//! - Clarification loops with multiple rounds
//! - Document review workflows
//! - Consensus building scenarios

mod office_test_helpers;

use anycowork::commands::mail::{get_mail_thread_messages, reply_to_mail, send_mail};
use anycowork::AppState;
use office_test_helpers::*;
use tauri::Manager;

/// Test 1.3: Forwarding and CC Pattern
///
/// Scenario: Message sent → Forwarded to another agent → Original sender looped in
/// Validates: Information flow across multiple parties
#[tokio::test]
async fn test_forwarding_pattern() -> Result<(), Box<dyn std::error::Error>> {
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
    let alice = create_test_agent(
        pool,
        "Alice Support",
        "💬",
        "Support agent handling customer inquiries",
    );
    let bob = create_test_agent(
        pool,
        "Bob Manager",
        "👔",
        "Support manager who handles escalations",
    );
    let charlie = create_test_agent(
        pool,
        "Charlie Expert",
        "🔬",
        "Technical expert for complex issues",
    );

    // Phase 1: Alice receives customer inquiry and emails Bob
    let initial = send_mail(
        state_handle.clone(),
        Some(alice.id.clone()),
        Some(bob.id.clone()),
        "Customer Question: API Rate Limits".to_string(),
        "Hi Bob,\n\nCustomer is asking about API rate limits for enterprise plan. Do we have documentation on this?\n\nAlice"
            .to_string(),
    )
    .await?;

    println!("✓ Alice sent inquiry to Bob");

    wait_for_mail_processing(state_handle.clone(), &bob.id, 1, 30).await?;
    println!("✓ Bob received inquiry");

    // Phase 2: Bob forwards to Charlie (simulated via new thread with context)
    let forward = send_mail(
        state_handle.clone(),
        Some(bob.id.clone()),
        Some(charlie.id.clone()),
        "FWD: Customer Question: API Rate Limits".to_string(),
        "Hi Charlie,\n\nAlice has a customer asking about enterprise API rate limits. Can you provide the technical details?\n\nForwarded from Alice's inquiry.\n\nBob"
            .to_string(),
    )
    .await?;

    println!("✓ Bob forwarded to Charlie");

    wait_for_mail_processing(state_handle.clone(), &charlie.id, 1, 30).await?;
    println!("✓ Charlie received forwarded message");

    // Phase 3: Charlie provides answer
    let _charlie_answer = reply_to_mail(
        state_handle.clone(),
        forward.id.clone(),
        Some(charlie.id.clone()),
        "Hi Bob,\n\nEnterprise plan has 10,000 requests/hour limit. Documentation is at docs.example.com/api-limits.\n\nCharlie"
            .to_string(),
    )
    .await?;

    println!("✓ Charlie provided answer");

    wait_for_mail_processing(state_handle.clone(), &bob.id, 2, 30).await?;
    println!("✓ Bob received Charlie's answer");

    // Phase 4: Bob replies to Alice with the info
    let _bob_answer = reply_to_mail(
        state_handle.clone(),
        initial.id.clone(),
        Some(bob.id.clone()),
        "Hi Alice,\n\nCharlie confirmed: Enterprise plan has 10,000 requests/hour. Docs at docs.example.com/api-limits.\n\nBob"
            .to_string(),
    )
    .await?;

    println!("✓ Bob replied to Alice with answer");

    wait_for_mail_processing(state_handle.clone(), &alice.id, 1, 30).await?;
    println!("✓ Alice received answer");

    // Verify information flow
    let initial_messages =
        get_mail_thread_messages(state_handle.clone(), initial.id).await?;
    let forward_messages =
        get_mail_thread_messages(state_handle.clone(), forward.id).await?;

    assert_eq!(
        initial_messages.len(),
        2,
        "Initial thread: Alice's question + Bob's answer"
    );
    assert_eq!(
        forward_messages.len(),
        2,
        "Forward thread: Bob's forward + Charlie's answer"
    );

    // Verify content flow
    assert!(initial_messages[0].content.contains("rate limits"));
    assert!(initial_messages[1].content.contains("10,000 requests"));
    assert!(forward_messages[1].content.contains("10,000 requests"));

    println!("\n✅ Test 1.3 PASSED: Forwarding pattern works correctly");
    Ok(())
}

/// Test 4.2: Clarification Loop
///
/// Scenario: Request → Ambiguous response → Clarification → Final answer
/// Validates: Multi-round clarification handling
#[tokio::test]
async fn test_clarification_loop() -> Result<(), Box<dyn std::error::Error>> {
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
    let requester = create_test_agent(
        pool,
        "Product Manager",
        "📊",
        "Product manager defining requirements",
    );
    let developer = create_test_agent(
        pool,
        "Developer Sam",
        "💻",
        "Developer implementing features",
    );

    // Round 1: Initial request
    let thread = send_mail(
        state_handle.clone(),
        Some(requester.id.clone()),
        Some(developer.id.clone()),
        "Feature Request: User Dashboard".to_string(),
        "Hi Sam,\n\nWe need a user dashboard. Can you build it?\n\nPM".to_string(),
    )
    .await?;

    println!("✓ PM sent initial request");

    wait_for_mail_processing(state_handle.clone(), &developer.id, 1, 30).await?;
    println!("✓ Developer received request");

    // Round 2: Developer asks for clarification
    let _clarification1 = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(developer.id.clone()),
        "Hi PM,\n\nCan you clarify what data should the dashboard display?\n\nSam".to_string(),
    )
    .await?;

    println!("✓ Developer requested clarification #1");

    wait_for_mail_processing(state_handle.clone(), &requester.id, 1, 30).await?;
    println!("✓ PM received clarification request #1");

    // Round 3: PM provides some details, but still incomplete
    let _partial_answer = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(requester.id.clone()),
        "Hi Sam,\n\nShow user activity and metrics.\n\nPM".to_string(),
    )
    .await?;

    println!("✓ PM provided partial answer");

    wait_for_mail_processing(state_handle.clone(), &developer.id, 2, 30).await?;
    println!("✓ Developer received partial answer");

    // Round 4: Developer asks for MORE clarification
    let _clarification2 = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(developer.id.clone()),
        "Hi PM,\n\nWhich specific metrics? Daily active users? Session duration? Revenue?\n\nSam"
            .to_string(),
    )
    .await?;

    println!("✓ Developer requested clarification #2");

    wait_for_mail_processing(state_handle.clone(), &requester.id, 2, 30).await?;
    println!("✓ PM received clarification request #2");

    // Round 5: PM provides complete details
    let _complete_answer = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(requester.id.clone()),
        "Hi Sam,\n\nShow: Daily active users, session duration, and total revenue. Use charts for visualization.\n\nPM"
            .to_string(),
    )
    .await?;

    println!("✓ PM provided complete answer");

    wait_for_mail_processing(state_handle.clone(), &developer.id, 3, 30).await?;
    println!("✓ Developer received complete answer");

    // Round 6: Developer confirms understanding
    let _confirmation = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(developer.id.clone()),
        "Hi PM,\n\nPerfect! I'll build the dashboard with those 3 metrics and charts. Starting today.\n\nSam"
            .to_string(),
    )
    .await?;

    println!("✓ Developer confirmed understanding");

    wait_for_mail_processing(state_handle.clone(), &requester.id, 3, 30).await?;
    println!("✓ PM received confirmation");

    // Verify multi-round clarification
    let messages = get_mail_thread_messages(state_handle.clone(), thread.id).await?;
    assert!(
        messages.len() >= 6,
        "Should have at least 6 messages in clarification loop"
    );

    // Verify content progression
    assert!(messages[0].content.contains("user dashboard"));
    assert!(messages[1].content.contains("clarify"));
    assert!(messages[2].content.contains("activity and metrics"));
    assert!(messages[3].content.contains("Which specific metrics"));
    assert!(messages[4].content.contains("Daily active users"));
    assert!(messages[5].content.contains("Starting today"));

    println!("\n✅ Test 4.2 PASSED: Multi-round clarification loop works");
    Ok(())
}

/// Test 5.1: Document Review Workflow
///
/// Scenario: Document shared → Multiple reviewers provide feedback → Author consolidates
/// Validates: Collaborative review process
#[tokio::test]
async fn test_document_review_workflow() -> Result<(), Box<dyn std::error::Error>> {
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
    let author = create_test_agent(
        pool,
        "Doc Author",
        "✍️",
        "Technical writer creating documentation",
    );
    let reviewer1 = create_test_agent(pool, "Tech Reviewer", "🔧", "Technical reviewer");
    let reviewer2 = create_test_agent(pool, "UX Reviewer", "🎨", "UX/content reviewer");

    // Phase 1: Author sends document for review
    let review1 = send_mail(
        state_handle.clone(),
        Some(author.id.clone()),
        Some(reviewer1.id.clone()),
        "Review Request: API Documentation Draft".to_string(),
        "Hi Tech Reviewer,\n\nPlease review the API documentation draft. Focus on technical accuracy.\n\nDoc Author"
            .to_string(),
    )
    .await?;

    let review2 = send_mail(
        state_handle.clone(),
        Some(author.id.clone()),
        Some(reviewer2.id.clone()),
        "Review Request: API Documentation Draft".to_string(),
        "Hi UX Reviewer,\n\nPlease review the API documentation draft. Focus on clarity and user experience.\n\nDoc Author"
            .to_string(),
    )
    .await?;

    println!("✓ Author sent review requests to 2 reviewers");

    wait_for_mail_processing(state_handle.clone(), &reviewer1.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &reviewer2.id, 1, 30).await?;
    println!("✓ Both reviewers received requests");

    // Phase 2: Reviewers provide feedback
    let _feedback1 = reply_to_mail(
        state_handle.clone(),
        review1.id.clone(),
        Some(reviewer1.id.clone()),
        "Hi Doc Author,\n\nTechnical review complete. Issues found:\n- Endpoint URL is incorrect\n- Missing error codes section\n\nTech Reviewer"
            .to_string(),
    )
    .await?;

    let _feedback2 = reply_to_mail(
        state_handle.clone(),
        review2.id.clone(),
        Some(reviewer2.id.clone()),
        "Hi Doc Author,\n\nUX review complete. Suggestions:\n- Add more examples\n- Simplify the introduction\n\nUX Reviewer"
            .to_string(),
    )
    .await?;

    println!("✓ Both reviewers provided feedback");

    wait_for_mail_processing(state_handle.clone(), &author.id, 2, 30).await?;
    println!("✓ Author received both feedback emails");

    // Phase 3: Author acknowledges and confirms changes
    let _ack1 = reply_to_mail(
        state_handle.clone(),
        review1.id.clone(),
        Some(author.id.clone()),
        "Hi Tech Reviewer,\n\nThanks! I've fixed the endpoint URL and added error codes section.\n\nDoc Author"
            .to_string(),
    )
    .await?;

    let _ack2 = reply_to_mail(
        state_handle.clone(),
        review2.id.clone(),
        Some(author.id.clone()),
        "Hi UX Reviewer,\n\nGreat feedback! I've added examples and simplified the intro.\n\nDoc Author"
            .to_string(),
    )
    .await?;

    println!("✓ Author acknowledged both reviews");

    wait_for_mail_processing(state_handle.clone(), &reviewer1.id, 2, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &reviewer2.id, 2, 30).await?;
    println!("✓ Reviewers received acknowledgments");

    // Verify review workflow
    let review1_messages =
        get_mail_thread_messages(state_handle.clone(), review1.id).await?;
    let review2_messages =
        get_mail_thread_messages(state_handle.clone(), review2.id).await?;

    assert_eq!(
        review1_messages.len(),
        3,
        "Tech review thread: request + feedback + ack"
    );
    assert_eq!(
        review2_messages.len(),
        3,
        "UX review thread: request + feedback + ack"
    );

    // Verify feedback content
    assert!(review1_messages[1].content.contains("Technical review"));
    assert!(review1_messages[1].content.contains("endpoint URL"));
    assert!(review2_messages[1].content.contains("UX review"));
    assert!(review2_messages[1].content.contains("examples"));

    // Verify author managed both review threads
    let author_inbox = get_inbox(state_handle.clone(), &author.id).await?;
    assert_eq!(
        author_inbox.len(),
        2,
        "Author should have 2 feedback emails"
    );

    let author_sent = get_sent_folder(state_handle.clone(), &author.id).await?;
    assert_eq!(
        author_sent.len(),
        4,
        "Author sent: 2 review requests + 2 acknowledgments"
    );

    println!("\n✅ Test 5.1 PASSED: Document review workflow works");
    Ok(())
}

/// Test 6.1: Consensus Building
///
/// Scenario: Proposal → Multiple opinions → Discussion → Final consensus
/// Validates: Multi-party decision making
#[tokio::test]
async fn test_consensus_building() -> Result<(), Box<dyn std::error::Error>> {
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
    let facilitator = create_test_agent(
        pool,
        "Team Lead",
        "👔",
        "Team lead facilitating architectural decisions",
    );
    let dev1 = create_test_agent(pool, "Dev A", "💻", "Backend specialist");
    let dev2 = create_test_agent(pool, "Dev B", "🖥️", "Frontend specialist");
    let dev3 = create_test_agent(pool, "Dev C", "🔧", "DevOps specialist");

    // Phase 1: Facilitator proposes architectural decision
    let proposal1 = send_mail(
        state_handle.clone(),
        Some(facilitator.id.clone()),
        Some(dev1.id.clone()),
        "Proposal: Microservices vs Monolith Architecture".to_string(),
        "Hi Dev A,\n\nShould we use microservices or monolith for the new project? Please share your opinion.\n\nTeam Lead"
            .to_string(),
    )
    .await?;

    let proposal2 = send_mail(
        state_handle.clone(),
        Some(facilitator.id.clone()),
        Some(dev2.id.clone()),
        "Proposal: Microservices vs Monolith Architecture".to_string(),
        "Hi Dev B,\n\nShould we use microservices or monolith for the new project? Please share your opinion.\n\nTeam Lead"
            .to_string(),
    )
    .await?;

    let proposal3 = send_mail(
        state_handle.clone(),
        Some(facilitator.id.clone()),
        Some(dev3.id.clone()),
        "Proposal: Microservices vs Monolith Architecture".to_string(),
        "Hi Dev C,\n\nShould we use microservices or monolith for the new project? Please share your opinion.\n\nTeam Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Facilitator sent proposals to 3 team members");

    wait_for_mail_processing(state_handle.clone(), &dev1.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &dev2.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &dev3.id, 1, 30).await?;
    println!("✓ All team members received proposal");

    // Phase 2: Team members share opinions
    let _opinion1 = reply_to_mail(
        state_handle.clone(),
        proposal1.id.clone(),
        Some(dev1.id.clone()),
        "Hi Team Lead,\n\nI prefer microservices for scalability, but it adds complexity.\n\nDev A"
            .to_string(),
    )
    .await?;

    let _opinion2 = reply_to_mail(
        state_handle.clone(),
        proposal2.id.clone(),
        Some(dev2.id.clone()),
        "Hi Team Lead,\n\nMonolith is simpler to start with. We can split later if needed.\n\nDev B"
            .to_string(),
    )
    .await?;

    let _opinion3 = reply_to_mail(
        state_handle.clone(),
        proposal3.id.clone(),
        Some(dev3.id.clone()),
        "Hi Team Lead,\n\nMonolith is easier to deploy and monitor initially.\n\nDev C"
            .to_string(),
    )
    .await?;

    println!("✓ All team members shared opinions");

    wait_for_mail_processing(state_handle.clone(), &facilitator.id, 3, 30).await?;
    println!("✓ Facilitator received all 3 opinions");

    // Phase 3: Facilitator synthesizes and announces consensus
    let _consensus1 = reply_to_mail(
        state_handle.clone(),
        proposal1.id.clone(),
        Some(facilitator.id.clone()),
        "Hi Dev A,\n\nTeam consensus: Start with monolith, architect for future microservices split.\n\nTeam Lead"
            .to_string(),
    )
    .await?;

    let _consensus2 = reply_to_mail(
        state_handle.clone(),
        proposal2.id.clone(),
        Some(facilitator.id.clone()),
        "Hi Dev B,\n\nTeam consensus: Start with monolith, architect for future microservices split.\n\nTeam Lead"
            .to_string(),
    )
    .await?;

    let _consensus3 = reply_to_mail(
        state_handle.clone(),
        proposal3.id.clone(),
        Some(facilitator.id.clone()),
        "Hi Dev C,\n\nTeam consensus: Start with monolith, architect for future microservices split.\n\nTeam Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Facilitator announced consensus to all");

    wait_for_mail_processing(state_handle.clone(), &dev1.id, 2, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &dev2.id, 2, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &dev3.id, 2, 30).await?;
    println!("✓ All team members received consensus");

    // Verify consensus building
    let proposal1_messages =
        get_mail_thread_messages(state_handle.clone(), proposal1.id).await?;
    let proposal2_messages =
        get_mail_thread_messages(state_handle.clone(), proposal2.id).await?;
    let proposal3_messages =
        get_mail_thread_messages(state_handle.clone(), proposal3.id).await?;

    assert_eq!(
        proposal1_messages.len(),
        3,
        "Thread 1: proposal + opinion + consensus"
    );
    assert_eq!(
        proposal2_messages.len(),
        3,
        "Thread 2: proposal + opinion + consensus"
    );
    assert_eq!(
        proposal3_messages.len(),
        3,
        "Thread 3: proposal + opinion + consensus"
    );

    // Verify consensus content
    assert!(proposal1_messages[2].content.contains("consensus"));
    assert!(proposal2_messages[2].content.contains("consensus"));
    assert!(proposal3_messages[2].content.contains("consensus"));

    // Verify facilitator managed all threads
    let facilitator_inbox = get_inbox(state_handle.clone(), &facilitator.id).await?;
    assert_eq!(
        facilitator_inbox.len(),
        3,
        "Facilitator should have 3 opinions"
    );

    let facilitator_sent = get_sent_folder(state_handle.clone(), &facilitator.id).await?;
    assert_eq!(
        facilitator_sent.len(),
        6,
        "Facilitator sent: 3 proposals + 3 consensus announcements"
    );

    println!("\n✅ Test 6.1 PASSED: Consensus building workflow works");
    Ok(())
}
