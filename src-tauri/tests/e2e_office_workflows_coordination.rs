//! End-to-end tests for coordination workflow patterns
//!
//! Tests Phase 2: Coordination Tests
//! - Meeting scheduling and negotiation
//! - Escalation workflows
//! - Research task distribution and synthesis

mod office_test_helpers;

use anycowork::commands::mail::{get_mail_thread_messages, reply_to_mail, send_mail};
use anycowork::AppState;
use office_test_helpers::*;
use tauri::Manager;

/// Test 2.1: Simple Meeting Scheduling
///
/// Scenario: Meeting invitation → rescheduling negotiation → final confirmation
/// Validates: Negotiation and iteration patterns
#[tokio::test]
async fn test_simple_meeting_scheduling() -> Result<(), Box<dyn std::error::Error>> {
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
    let alice = create_test_agent(pool, "Alice PM", "📋", "Project manager organizing meetings");
    let bob = create_test_agent(
        pool,
        "Bob Engineer",
        "⚙️",
        "Senior engineer with a busy schedule",
    );

    // Alice proposes a meeting
    let meeting_thread = send_mail(
        state_handle.clone(),
        Some(alice.id.clone()),
        Some(bob.id.clone()),
        "Meeting Request: Sprint Planning".to_string(),
        "Hi Bob,\n\nCan we meet tomorrow at 2 PM to discuss sprint planning?\n\nAlice".to_string(),
    )
    .await?;

    println!("✓ Alice sent meeting invitation");

    // Wait for Bob to receive
    wait_for_mail_processing(state_handle.clone(), &bob.id, 1, 30).await?;
    println!("✓ Bob received invitation");

    // Bob proposes reschedule
    let _reschedule = reply_to_mail(
        state_handle.clone(),
        meeting_thread.id.clone(),
        Some(bob.id.clone()),
        "Hi Alice,\n\nI have a conflict at 2 PM. Can we do 4 PM instead?\n\nBob".to_string(),
    )
    .await?;

    println!("✓ Bob proposed reschedule");

    // Wait for Alice to receive reschedule request
    wait_for_mail_processing(state_handle.clone(), &alice.id, 1, 30).await?;
    println!("✓ Alice received reschedule request");

    // Alice confirms new time
    let _confirmation = reply_to_mail(
        state_handle.clone(),
        meeting_thread.id.clone(),
        Some(alice.id.clone()),
        "Hi Bob,\n\n4 PM works great! See you then.\n\nAlice".to_string(),
    )
    .await?;

    println!("✓ Alice confirmed new time");

    // Wait for Bob to receive confirmation
    wait_for_mail_processing(state_handle.clone(), &bob.id, 2, 30).await?;
    println!("✓ Bob received confirmation");

    // Verify thread has complete negotiation history
    let messages =
        get_mail_thread_messages(state_handle.clone(), meeting_thread.id).await?;
    assert_eq!(
        messages.len(),
        3,
        "Meeting thread should have: invitation + reschedule + confirmation"
    );

    // Verify content progression
    assert!(messages[0].content.contains("2 PM"));
    assert!(messages[1].content.contains("4 PM"));
    assert!(messages[2].content.contains("works great"));

    // Verify both parties have the thread
    let alice_inbox = get_inbox(state_handle.clone(), &alice.id).await?;
    let bob_inbox = get_inbox(state_handle.clone(), &bob.id).await?;

    assert!(
        alice_inbox.len() >= 1,
        "Alice should have the meeting thread in inbox"
    );
    assert_eq!(bob_inbox.len(), 1, "Bob should have the meeting thread");

    println!("\n✅ Test 2.1 PASSED: Meeting scheduling negotiation works");
    Ok(())
}

/// Test 3.2: Escalation Workflow
///
/// Scenario: Developer blocked → PM forwards to tech lead → decision → resolution
/// Validates: Hierarchical communication chains
#[tokio::test]
async fn test_escalation_workflow() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // Create agents representing hierarchy
    let dev = create_test_agent(
        pool,
        "Dev Junior",
        "💻",
        "Junior developer who needs help with architecture decisions",
    );
    let pm = create_test_agent(
        pool,
        "PM Sarah",
        "📋",
        "Project manager who coordinates between team members",
    );
    let tech_lead = create_test_agent(
        pool,
        "Tech Lead Mike",
        "👔",
        "Technical lead with architecture expertise",
    );

    // Step 1: Dev reports blocker to PM
    let blocker_thread = send_mail(
        state_handle.clone(),
        Some(dev.id.clone()),
        Some(pm.id.clone()),
        "Blocked: Database Schema Decision Needed".to_string(),
        "Hi Sarah,\n\nI'm blocked on the user service. Should we use PostgreSQL or MySQL for the database? This is impacting my timeline.\n\nDev"
            .to_string(),
    )
    .await?;

    println!("✓ Dev reported blocker to PM");

    wait_for_mail_processing(state_handle.clone(), &pm.id, 1, 30).await?;
    println!("✓ PM received blocker report");

    // Step 2: PM escalates to tech lead
    let escalation_thread = send_mail(
        state_handle.clone(),
        Some(pm.id.clone()),
        Some(tech_lead.id.clone()),
        "Architecture Decision Needed: Database Choice".to_string(),
        "Hi Mike,\n\nDev is blocked on database choice for user service. Can you provide guidance on PostgreSQL vs MySQL?\n\nSarah"
            .to_string(),
    )
    .await?;

    println!("✓ PM escalated to tech lead");

    wait_for_mail_processing(state_handle.clone(), &tech_lead.id, 1, 30).await?;
    println!("✓ Tech lead received escalation");

    // Step 3: Tech lead provides decision
    let _decision = reply_to_mail(
        state_handle.clone(),
        escalation_thread.id.clone(),
        Some(tech_lead.id.clone()),
        "Hi Sarah,\n\nLet's go with PostgreSQL for better JSON support and scalability. I'll document this in our architecture guide.\n\nMike"
            .to_string(),
    )
    .await?;

    println!("✓ Tech lead provided decision");

    wait_for_mail_processing(state_handle.clone(), &pm.id, 2, 30).await?;
    println!("✓ PM received decision");

    // Step 4: PM communicates resolution back to dev
    let _resolution = reply_to_mail(
        state_handle.clone(),
        blocker_thread.id.clone(),
        Some(pm.id.clone()),
        "Hi Dev,\n\nMike confirmed we should use PostgreSQL for the user service. You're unblocked now!\n\nSarah"
            .to_string(),
    )
    .await?;

    println!("✓ PM sent resolution to dev");

    wait_for_mail_processing(state_handle.clone(), &dev.id, 1, 30).await?;
    println!("✓ Dev received resolution");

    // Verify complete escalation chain
    let blocker_messages =
        get_mail_thread_messages(state_handle.clone(), blocker_thread.id).await?;
    let escalation_messages =
        get_mail_thread_messages(state_handle.clone(), escalation_thread.id).await?;

    assert_eq!(
        blocker_messages.len(),
        2,
        "Blocker thread: initial report + resolution"
    );
    assert_eq!(
        escalation_messages.len(),
        2,
        "Escalation thread: escalation + decision"
    );

    // Verify escalation content
    assert!(blocker_messages[0].content.contains("blocked"));
    assert!(blocker_messages[1].content.contains("unblocked"));
    assert!(escalation_messages[0].content.contains("guidance"));
    assert!(escalation_messages[1].content.contains("PostgreSQL"));

    // Verify all parties have appropriate emails
    let pm_inbox = get_inbox(state_handle.clone(), &pm.id).await?;
    assert_eq!(
        pm_inbox.len(),
        2,
        "PM should have 2 emails: blocker report + tech lead decision"
    );

    let dev_inbox = get_inbox(state_handle.clone(), &dev.id).await?;
    assert_eq!(dev_inbox.len(), 1, "Dev should have resolution email");

    let tech_lead_inbox = get_inbox(state_handle.clone(), &tech_lead.id).await?;
    assert_eq!(
        tech_lead_inbox.len(),
        1,
        "Tech lead should have escalation email"
    );

    println!("\n✅ Test 3.2 PASSED: Escalation workflow with hierarchy works");
    Ok(())
}

/// Test 4.1: Research Task Distribution
///
/// Scenario: Coordinator distributes research → Specialists work in parallel → Results synthesized
/// Validates: Parallel work collection and synthesis
#[tokio::test]
async fn test_research_task_distribution() -> Result<(), Box<dyn std::error::Error>> {
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
    let coordinator = create_test_agent(
        pool,
        "Research Coordinator",
        "🔬",
        "Coordinates research projects and synthesizes findings",
    );
    let specialist_a = create_test_agent(
        pool,
        "AI Specialist",
        "🤖",
        "Expert in artificial intelligence and machine learning",
    );
    let specialist_b = create_test_agent(
        pool,
        "Security Specialist",
        "🔒",
        "Expert in cybersecurity and data protection",
    );
    let specialist_c = create_test_agent(
        pool,
        "UX Specialist",
        "🎨",
        "Expert in user experience and interface design",
    );

    // Coordinator sends research requests
    let task_ai = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(specialist_a.id.clone()),
        "Research Task: AI Implementation Options".to_string(),
        "Hi AI Specialist,\n\nPlease research implementation options for AI-powered features in our app.\n\nCoordinator"
            .to_string(),
    )
    .await?;

    let task_security = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(specialist_b.id.clone()),
        "Research Task: Security Best Practices".to_string(),
        "Hi Security Specialist,\n\nPlease research security best practices for our new authentication system.\n\nCoordinator"
            .to_string(),
    )
    .await?;

    let task_ux = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(specialist_c.id.clone()),
        "Research Task: UX Patterns".to_string(),
        "Hi UX Specialist,\n\nPlease research modern UX patterns for dashboard interfaces.\n\nCoordinator"
            .to_string(),
    )
    .await?;

    println!("✓ Coordinator distributed 3 research tasks");

    // Wait for all to receive
    wait_for_mail_processing(state_handle.clone(), &specialist_a.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &specialist_b.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &specialist_c.id, 1, 30).await?;
    println!("✓ All specialists received tasks");

    // Specialists submit findings
    let _finding_ai = reply_to_mail(
        state_handle.clone(),
        task_ai.id.clone(),
        Some(specialist_a.id.clone()),
        "Hi Coordinator,\n\nI recommend using GPT-4 API for natural language features. Cost-effective and powerful.\n\nAI Specialist"
            .to_string(),
    )
    .await?;

    let _finding_security = reply_to_mail(
        state_handle.clone(),
        task_security.id.clone(),
        Some(specialist_b.id.clone()),
        "Hi Coordinator,\n\nRecommend implementing OAuth 2.0 with JWT tokens and rate limiting.\n\nSecurity Specialist"
            .to_string(),
    )
    .await?;

    let _finding_ux = reply_to_mail(
        state_handle.clone(),
        task_ux.id.clone(),
        Some(specialist_c.id.clone()),
        "Hi Coordinator,\n\nModern dashboards should use card-based layouts with data visualization widgets.\n\nUX Specialist"
            .to_string(),
    )
    .await?;

    println!("✓ All specialists submitted findings");

    // Wait for coordinator to receive all findings
    wait_for_mail_processing(state_handle.clone(), &coordinator.id, 3, 30).await?;
    println!("✓ Coordinator received all 3 findings");

    // Verify coordinator's inbox has all findings
    let coordinator_inbox = get_inbox(state_handle.clone(), &coordinator.id).await?;
    assert_eq!(
        coordinator_inbox.len(),
        3,
        "Coordinator should have 3 findings in inbox"
    );

    // Verify coordinator's sent folder has all tasks
    let coordinator_sent = get_sent_folder(state_handle.clone(), &coordinator.id).await?;
    assert_eq!(
        coordinator_sent.len(),
        3,
        "Coordinator should have 3 tasks in sent folder"
    );

    // Verify thread structure for each research task
    let ai_messages = get_mail_thread_messages(state_handle.clone(), task_ai.id).await?;
    let security_messages =
        get_mail_thread_messages(state_handle.clone(), task_security.id).await?;
    let ux_messages = get_mail_thread_messages(state_handle.clone(), task_ux.id).await?;

    assert_eq!(ai_messages.len(), 2, "AI task: request + finding");
    assert_eq!(
        security_messages.len(),
        2,
        "Security task: request + finding"
    );
    assert_eq!(ux_messages.len(), 2, "UX task: request + finding");

    // Verify findings content
    assert!(ai_messages[1].content.contains("GPT-4"));
    assert!(security_messages[1].content.contains("OAuth"));
    assert!(ux_messages[1].content.contains("card-based"));

    println!("\n✅ Test 4.1 PASSED: Research task distribution and collection works");
    Ok(())
}
