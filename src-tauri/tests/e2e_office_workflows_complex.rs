//! End-to-end tests for complex multi-agent workflow patterns
//!
//! Tests Phase 3: Complex Workflows
//! - Cross-team meeting coordination with 6+ agents
//! - Parallel task coordination with convergence
//! - Feature launch coordination across departments

mod office_test_helpers;

use anycowork::commands::mail::{get_mail_thread_messages, reply_to_mail, send_mail};
use anycowork::AppState;
use office_test_helpers::*;
use tauri::Manager;

/// Test 2.2: Cross-Team Meeting Coordination
///
/// Scenario: 6 agents across 2 teams → Parallel polling → Final confirmation
/// Validates: Complex communication patterns at scale
#[tokio::test]
async fn test_cross_team_meeting_coordination() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // Create organizer
    let organizer = create_test_agent(
        pool,
        "Meeting Organizer",
        "📅",
        "Organizes cross-team meetings and events",
    );

    // Create Engineering team (3 members)
    let eng1 = create_test_agent(pool, "Eng Alice", "⚙️", "Backend engineer");
    let eng2 = create_test_agent(pool, "Eng Bob", "💻", "Frontend engineer");
    let eng3 = create_test_agent(pool, "Eng Carol", "🔧", "DevOps engineer");

    // Create Product team (3 members)
    let prod1 = create_test_agent(pool, "Product Dan", "📊", "Product manager");
    let prod2 = create_test_agent(pool, "Product Eve", "📈", "Product analyst");
    let prod3 = create_test_agent(pool, "Product Frank", "🎯", "Product designer");

    // Organizer sends meeting invitations to all 6 people
    let invitations = vec![
        (&eng1, "Eng Alice"),
        (&eng2, "Eng Bob"),
        (&eng3, "Eng Carol"),
        (&prod1, "Product Dan"),
        (&prod2, "Product Eve"),
        (&prod3, "Product Frank"),
    ];

    let mut threads = Vec::new();
    for (agent, name) in &invitations {
        let thread = send_mail(
            state_handle.clone(),
            Some(organizer.id.clone()),
            Some(agent.id.clone()),
            "Cross-Team Sync: Q1 Planning".to_string(),
            format!("Hi {},\n\nCan you join our Q1 planning meeting next Tuesday at 10 AM?\n\nOrganizer", name),
        )
        .await?;
        threads.push(thread);
    }

    println!("✓ Sent invitations to 6 team members");

    // Wait for all to receive
    for (agent, _) in &invitations {
        wait_for_mail_processing(state_handle.clone(), &agent.id, 1, 30).await?;
    }
    println!("✓ All 6 members received invitations");

    // 4 people confirm, 2 propose reschedule
    let _confirm1 = reply_to_mail(
        state_handle.clone(),
        threads[0].id.clone(),
        Some(eng1.id.clone()),
        "I can make it! See you there.\n\nAlice".to_string(),
    )
    .await?;

    let _confirm2 = reply_to_mail(
        state_handle.clone(),
        threads[1].id.clone(),
        Some(eng2.id.clone()),
        "Confirmed!\n\nBob".to_string(),
    )
    .await?;

    let _reschedule1 = reply_to_mail(
        state_handle.clone(),
        threads[2].id.clone(),
        Some(eng3.id.clone()),
        "I have a conflict. Can we do 2 PM instead?\n\nCarol".to_string(),
    )
    .await?;

    let _confirm3 = reply_to_mail(
        state_handle.clone(),
        threads[3].id.clone(),
        Some(prod1.id.clone()),
        "Works for me!\n\nDan".to_string(),
    )
    .await?;

    let _reschedule2 = reply_to_mail(
        state_handle.clone(),
        threads[4].id.clone(),
        Some(prod2.id.clone()),
        "I'm out of office Tuesday. Can we do Wednesday?\n\nEve".to_string(),
    )
    .await?;

    let _confirm4 = reply_to_mail(
        state_handle.clone(),
        threads[5].id.clone(),
        Some(prod3.id.clone()),
        "I'll be there.\n\nFrank".to_string(),
    )
    .await?;

    println!("✓ Received 4 confirmations and 2 reschedule requests");

    // Wait for organizer to receive all 6 responses
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    wait_for_mail_processing(state_handle.clone(), &organizer.id, 6, 45).await?;
    println!("✓ Organizer received all 6 responses");

    // Verify organizer has 6 emails in inbox (all responses)
    let organizer_inbox = get_inbox(state_handle.clone(), &organizer.id).await?;
    assert!(
        organizer_inbox.len() >= 6,
        "Organizer should have at least 6 responses"
    );

    // Verify organizer sent 6 invitations
    let organizer_sent = get_sent_folder(state_handle.clone(), &organizer.id).await?;
    assert_eq!(organizer_sent.len(), 6, "Should have sent 6 invitations");

    // Verify thread structure - each should have 2 messages (invitation + response)
    for thread in &threads {
        let messages =
            get_mail_thread_messages(state_handle.clone(), thread.id.clone()).await?;
        assert_eq!(
            messages.len(),
            2,
            "Each thread should have invitation + response"
        );
    }

    println!("\n✅ Test 2.2 PASSED: Cross-team coordination with 6 agents works");
    Ok(())
}

/// Test 3.3: Parallel Task Coordination
///
/// Scenario: Independent parallel work → Integration → Testing → Bug fix cycle
/// Validates: Convergence of parallel work streams
#[tokio::test]
async fn test_parallel_task_coordination() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // Create team
    let coordinator = create_test_agent(
        pool,
        "Tech Lead",
        "👔",
        "Technical lead coordinating feature development",
    );
    let backend_dev = create_test_agent(pool, "Backend Dev", "💻", "Backend developer");
    let frontend_dev = create_test_agent(pool, "Frontend Dev", "🖥️", "Frontend developer");
    let qa_engineer = create_test_agent(pool, "QA Engineer", "🧪", "Quality assurance engineer");

    // Phase 1: Assign parallel tasks
    let backend_task = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(backend_dev.id.clone()),
        "Task: Build User Profile API".to_string(),
        "Hi Backend Dev,\n\nBuild the user profile API with GET, POST, PUT endpoints.\n\nTech Lead"
            .to_string(),
    )
    .await?;

    let frontend_task = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(frontend_dev.id.clone()),
        "Task: Build User Profile UI".to_string(),
        "Hi Frontend Dev,\n\nBuild the user profile UI component.\n\nTech Lead".to_string(),
    )
    .await?;

    println!("✓ Assigned parallel tasks to backend and frontend");

    wait_for_mail_processing(state_handle.clone(), &backend_dev.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &frontend_dev.id, 1, 30).await?;
    println!("✓ Both developers received tasks");

    // Phase 2: Developers report completion
    let _backend_complete = reply_to_mail(
        state_handle.clone(),
        backend_task.id.clone(),
        Some(backend_dev.id.clone()),
        "Hi Tech Lead,\n\nAPI is complete and deployed to staging.\n\nBackend Dev".to_string(),
    )
    .await?;

    let _frontend_complete = reply_to_mail(
        state_handle.clone(),
        frontend_task.id.clone(),
        Some(frontend_dev.id.clone()),
        "Hi Tech Lead,\n\nUI component is ready for integration.\n\nFrontend Dev".to_string(),
    )
    .await?;

    println!("✓ Both developers reported completion");

    wait_for_mail_processing(state_handle.clone(), &coordinator.id, 2, 30).await?;
    println!("✓ Coordinator received completion reports");

    // Phase 3: Request QA testing
    let qa_task = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(qa_engineer.id.clone()),
        "Task: Test User Profile Feature".to_string(),
        "Hi QA Engineer,\n\nBackend and frontend are ready. Please test the user profile feature on staging.\n\nTech Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Requested QA testing");

    wait_for_mail_processing(state_handle.clone(), &qa_engineer.id, 1, 30).await?;
    println!("✓ QA received testing task");

    // Phase 4: QA finds bug and reports
    let _bug_report = reply_to_mail(
        state_handle.clone(),
        qa_task.id.clone(),
        Some(qa_engineer.id.clone()),
        "Hi Tech Lead,\n\nFound a bug: profile photo upload fails. Backend returns 500 error.\n\nQA Engineer"
            .to_string(),
    )
    .await?;

    println!("✓ QA reported bug");

    wait_for_mail_processing(state_handle.clone(), &coordinator.id, 3, 30).await?;
    println!("✓ Coordinator received bug report");

    // Phase 5: Assign bug fix
    let bug_fix = send_mail(
        state_handle.clone(),
        Some(coordinator.id.clone()),
        Some(backend_dev.id.clone()),
        "Bug Fix: Profile Photo Upload 500 Error".to_string(),
        "Hi Backend Dev,\n\nQA found a 500 error on photo upload. Can you fix this?\n\nTech Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Assigned bug fix");

    wait_for_mail_processing(state_handle.clone(), &backend_dev.id, 2, 30).await?;
    println!("✓ Backend dev received bug assignment");

    // Phase 6: Bug fixed
    let _bug_fixed = reply_to_mail(
        state_handle.clone(),
        bug_fix.id.clone(),
        Some(backend_dev.id.clone()),
        "Hi Tech Lead,\n\nBug fixed! Issue was file size validation. Deployed to staging.\n\nBackend Dev"
            .to_string(),
    )
    .await?;

    println!("✓ Bug fixed");

    wait_for_mail_processing(state_handle.clone(), &coordinator.id, 4, 30).await?;
    println!("✓ Coordinator received bug fix confirmation");

    // Verify complete workflow
    let coordinator_inbox = get_inbox(state_handle.clone(), &coordinator.id).await?;
    assert!(
        coordinator_inbox.len() >= 4,
        "Coordinator should have: backend done + frontend done + bug report + bug fixed"
    );

    // Verify thread structure
    let backend_messages =
        get_mail_thread_messages(state_handle.clone(), backend_task.id).await?;
    let frontend_messages =
        get_mail_thread_messages(state_handle.clone(), frontend_task.id).await?;
    let qa_messages = get_mail_thread_messages(state_handle.clone(), qa_task.id).await?;
    let bugfix_messages =
        get_mail_thread_messages(state_handle.clone(), bug_fix.id).await?;

    assert_eq!(backend_messages.len(), 2, "Backend task + completion");
    assert_eq!(frontend_messages.len(), 2, "Frontend task + completion");
    assert_eq!(qa_messages.len(), 2, "QA task + bug report");
    assert_eq!(bugfix_messages.len(), 2, "Bug assignment + fix");

    println!("\n✅ Test 3.3 PASSED: Parallel task coordination with convergence works");
    Ok(())
}

/// Test 7.1: Feature Launch Coordination
///
/// Scenario: Cross-functional coordination (Eng, Marketing, Support) → Dependencies → Launch
/// Validates: Orchestration at scale across departments
#[tokio::test]
async fn test_feature_launch_coordination() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let state_handle = app.state::<AppState>();
    let pool = &state.db_pool;

    // Create cross-functional team
    let launch_manager = create_test_agent(
        pool,
        "Launch Manager",
        "🚀",
        "Manages product launches across departments",
    );
    let engineering = create_test_agent(pool, "Engineering Lead", "⚙️", "Engineering team lead");
    let marketing = create_test_agent(
        pool,
        "Marketing Lead",
        "📣",
        "Marketing team lead",
    );
    let support = create_test_agent(pool, "Support Lead", "💬", "Customer support lead");

    // Phase 1: Launch manager kicks off coordination
    let eng_task = send_mail(
        state_handle.clone(),
        Some(launch_manager.id.clone()),
        Some(engineering.id.clone()),
        "Feature Launch: New Analytics Dashboard".to_string(),
        "Hi Engineering Lead,\n\nWe're launching the analytics dashboard next week. Please confirm feature completion by Friday.\n\nLaunch Manager"
            .to_string(),
    )
    .await?;

    let marketing_task = send_mail(
        state_handle.clone(),
        Some(launch_manager.id.clone()),
        Some(marketing.id.clone()),
        "Feature Launch: Marketing Campaign Needed".to_string(),
        "Hi Marketing Lead,\n\nPlease prepare launch campaign for analytics dashboard.\n\nLaunch Manager"
            .to_string(),
    )
    .await?;

    let support_task = send_mail(
        state_handle.clone(),
        Some(launch_manager.id.clone()),
        Some(support.id.clone()),
        "Feature Launch: Support Documentation Needed".to_string(),
        "Hi Support Lead,\n\nPlease prepare support docs and FAQs for analytics dashboard.\n\nLaunch Manager"
            .to_string(),
    )
    .await?;

    println!("✓ Launch manager coordinated with 3 departments");

    // Wait for all to receive
    wait_for_mail_processing(state_handle.clone(), &engineering.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &marketing.id, 1, 30).await?;
    wait_for_mail_processing(state_handle.clone(), &support.id, 1, 30).await?;
    println!("✓ All departments received launch tasks");

    // Phase 2: Engineering confirms completion
    let _eng_ready = reply_to_mail(
        state_handle.clone(),
        eng_task.id.clone(),
        Some(engineering.id.clone()),
        "Hi Launch Manager,\n\nFeature is complete and tested. Ready for launch!\n\nEngineering Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Engineering confirmed readiness");

    wait_for_mail_processing(state_handle.clone(), &launch_manager.id, 1, 30).await?;

    // Phase 3: Marketing needs dependency
    let _marketing_question = reply_to_mail(
        state_handle.clone(),
        marketing_task.id.clone(),
        Some(marketing.id.clone()),
        "Hi Launch Manager,\n\nI need screenshots of the dashboard for the campaign. Can engineering provide?\n\nMarketing Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Marketing requested screenshots");

    wait_for_mail_processing(state_handle.clone(), &launch_manager.id, 2, 30).await?;

    // Phase 4: Launch manager coordinates dependency
    let _screenshot_request = send_mail(
        state_handle.clone(),
        Some(launch_manager.id.clone()),
        Some(engineering.id.clone()),
        "Request: Dashboard Screenshots for Marketing".to_string(),
        "Hi Engineering Lead,\n\nMarketing needs dashboard screenshots for the launch campaign. Can you provide?\n\nLaunch Manager"
            .to_string(),
    )
    .await?;

    println!("✓ Requested screenshots from engineering");

    wait_for_mail_processing(state_handle.clone(), &engineering.id, 2, 30).await?;

    // Phase 5: Engineering provides screenshots
    let _screenshots_ready = reply_to_mail(
        state_handle.clone(),
        eng_task.id.clone(),
        Some(engineering.id.clone()),
        "Hi Launch Manager,\n\nScreenshots are ready in the shared drive. Marketing can access them now.\n\nEngineering Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Engineering provided screenshots");

    wait_for_mail_processing(state_handle.clone(), &launch_manager.id, 3, 30).await?;

    // Phase 6: Support confirms docs ready
    let _support_ready = reply_to_mail(
        state_handle.clone(),
        support_task.id.clone(),
        Some(support.id.clone()),
        "Hi Launch Manager,\n\nSupport docs and FAQs are published. Team is trained.\n\nSupport Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Support confirmed documentation ready");

    wait_for_mail_processing(state_handle.clone(), &launch_manager.id, 4, 30).await?;

    // Phase 7: Marketing confirms campaign ready
    let _marketing_ready = reply_to_mail(
        state_handle.clone(),
        marketing_task.id.clone(),
        Some(marketing.id.clone()),
        "Hi Launch Manager,\n\nCampaign is ready! Blog post and social media scheduled.\n\nMarketing Lead"
            .to_string(),
    )
    .await?;

    println!("✓ Marketing confirmed campaign ready");

    wait_for_mail_processing(state_handle.clone(), &launch_manager.id, 5, 30).await?;

    // Verify launch manager orchestrated everything
    let launch_manager_inbox = get_inbox(state_handle.clone(), &launch_manager.id).await?;
    assert!(
        launch_manager_inbox.len() >= 5,
        "Launch manager should have all status updates"
    );

    let launch_manager_sent = get_sent_folder(state_handle.clone(), &launch_manager.id).await?;
    assert!(
        launch_manager_sent.len() >= 4,
        "Launch manager sent: 3 initial tasks + 1 screenshot request"
    );

    // Verify thread structures
    let eng_messages = get_mail_thread_messages(state_handle.clone(), eng_task.id).await?;
    let marketing_messages =
        get_mail_thread_messages(state_handle.clone(), marketing_task.id).await?;
    let support_messages =
        get_mail_thread_messages(state_handle.clone(), support_task.id).await?;

    assert!(
        eng_messages.len() >= 2,
        "Engineering thread has multiple updates"
    );
    assert!(
        marketing_messages.len() >= 2,
        "Marketing thread has question + confirmation"
    );
    assert_eq!(
        support_messages.len(),
        2,
        "Support thread: task + confirmation"
    );

    println!("\n✅ Test 7.1 PASSED: Cross-functional feature launch coordination works");
    Ok(())
}
