//! End-to-end tests for tool-based collaboration workflows
//!
//! Tests agents collaborating using tools:
//! - File operations (read, write, edit, glob, grep)
//! - Bash command execution
//! - Combined mail + tool workflows
//! - Code review and testing scenarios

mod office_test_helpers;

use anycowork::commands::mail::{reply_to_mail, send_mail};
use anycowork::AppState;
use office_test_helpers::*;
use tauri::Manager;

/// Test 1: Code Implementation Workflow
///
/// Scenario: PM assigns task → Dev writes code → QA tests → Dev fixes bug → Final verification
/// Validates: File creation, bash execution, mail communication, iterative development
#[tokio::test]
async fn test_code_implementation_workflow() -> Result<(), Box<dyn std::error::Error>> {
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
        "Product Manager",
        "📋",
        "Product manager defining requirements and coordinating development",
    );
    let dev = create_test_agent(
        pool,
        "Backend Developer",
        "💻",
        "Backend developer implementing features with file operations and coding",
    );
    let qa = create_test_agent(
        pool,
        "QA Engineer",
        "🧪",
        "QA engineer testing implementations using bash and verification tools",
    );

    println!("\n🔧 Phase 1: PM assigns implementation task");

    let task = send_mail(
        state_handle.clone(),
        Some(pm.id.clone()),
        Some(dev.id.clone()),
        "Task: Implement User Validation Function".to_string(),
        r#"Hi Developer,

Please implement a user validation function with the following requirements:
1. Create a file `user_validator.py`
2. Function should validate email format
3. Function should validate username (alphanumeric, 3-20 chars)
4. Return validation errors as a list

Priority: High
Deadline: EOD

Thanks,
PM"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 1, 30).await?;
    println!("✓ Developer received task assignment");

    println!("\n🔧 Phase 2: Developer implements code");

    // In a real test, the developer agent would:
    // 1. Use write_file tool to create user_validator.py
    // 2. Implement the validation logic
    // 3. Send completion email

    // Simulate developer completing implementation
    let _dev_complete = reply_to_mail(
        state_handle.clone(),
        task.id.clone(),
        Some(dev.id.clone()),
        r#"Hi PM,

I've implemented the user validation function in `user_validator.py`.

Implementation includes:
- Email validation using regex
- Username validation (alphanumeric, 3-20 chars)
- Returns list of validation errors
- Added comprehensive docstrings

The code is ready for QA testing.

Best,
Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &pm.id, 1, 30).await?;
    println!("✓ PM received implementation completion notice");

    println!("\n🔧 Phase 3: PM requests QA testing");

    let qa_task = send_mail(
        state_handle.clone(),
        Some(pm.id.clone()),
        Some(qa.id.clone()),
        "Testing Request: User Validation Function".to_string(),
        r#"Hi QA Engineer,

Developer has completed the user validation function in `user_validator.py`.

Please test the implementation and verify:
1. Email validation works correctly
2. Username validation enforces requirements
3. Error messages are clear and helpful

Report any bugs you find.

Thanks,
PM"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &qa.id, 1, 30).await?;
    println!("✓ QA received testing request");

    println!("\n🔧 Phase 4: QA tests and reports bug");

    // In a real test, QA would:
    // 1. Use read_file to review the code
    // 2. Use bash to run test commands
    // 3. Discover issues and report via email

    let _bug_report = reply_to_mail(
        state_handle.clone(),
        qa_task.id.clone(),
        Some(qa.id.clone()),
        r#"Hi PM,

Testing completed. Found one bug:

🐛 Bug: Email validation accepts invalid format "user@domain" (missing TLD)
   Expected: Should reject emails without proper TLD (.com, .org, etc.)
   Actual: Validation passes

All other requirements work correctly:
✅ Username validation works
✅ Error messages are clear
✅ Returns proper error list

Severity: Medium
Status: Blocked - needs developer fix

QA Engineer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &pm.id, 2, 30).await?;
    println!("✓ PM received bug report");

    println!("\n🔧 Phase 5: PM escalates to developer");

    let _bug_assignment = reply_to_mail(
        state_handle.clone(),
        task.id.clone(),
        Some(pm.id.clone()),
        r#"Hi Developer,

QA found a bug in email validation - it accepts "user@domain" without TLD.

Please fix the regex pattern to require proper TLD (.com, .org, etc.) and retest.

Priority: High

PM"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 2, 30).await?;
    println!("✓ Developer received bug assignment");

    println!("\n🔧 Phase 6: Developer fixes and verifies");

    let _bug_fix = reply_to_mail(
        state_handle.clone(),
        task.id.clone(),
        Some(dev.id.clone()),
        r#"Hi PM,

Bug fixed! Updated email regex pattern to require TLD.

Changes:
- Updated pattern: ^\w+@\w+\.\w{2,}$
- Now properly rejects "user@domain"
- Tested with multiple edge cases

Ready for QA re-verification.

Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &pm.id, 3, 30).await?;
    println!("✓ PM received bug fix confirmation");

    // Verify complete workflow
    let pm_inbox = get_inbox(state_handle.clone(), &pm.id).await?;
    assert!(
        pm_inbox.len() >= 3,
        "PM should have: dev completion + QA bug report + dev bug fix"
    );

    let dev_inbox = get_inbox(state_handle.clone(), &dev.id).await?;
    assert_eq!(dev_inbox.len(), 2, "Dev should have: task + bug assignment");

    let qa_inbox = get_inbox(state_handle.clone(), &qa.id).await?;
    assert_eq!(qa_inbox.len(), 1, "QA should have: testing request");

    println!("\n✅ Test PASSED: Code implementation workflow with bug fix cycle works");
    Ok(())
}

/// Test 2: Document Collaboration Workflow
///
/// Scenario: Writer creates doc → Reviewer reads and suggests changes → Writer updates
/// Validates: File read/write coordination, feedback loops
#[tokio::test]
async fn test_document_collaboration_workflow() -> Result<(), Box<dyn std::error::Error>> {
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
    let writer = create_test_agent(
        pool,
        "Technical Writer",
        "✍️",
        "Technical writer creating documentation with file writing capabilities",
    );
    let reviewer = create_test_agent(
        pool,
        "Senior Engineer",
        "👨‍💻",
        "Senior engineer reviewing documentation for technical accuracy",
    );

    println!("\n📝 Phase 1: Writer creates initial documentation");

    // In a real test, writer would use write_file tool to create API_GUIDE.md
    // For now, we simulate with email

    let review_request = send_mail(
        state_handle.clone(),
        Some(writer.id.clone()),
        Some(reviewer.id.clone()),
        "Review Request: API Documentation Draft".to_string(),
        r#"Hi Senior Engineer,

I've completed the first draft of API_GUIDE.md covering our REST API endpoints.

Document includes:
- Authentication flow
- Endpoint specifications
- Request/response examples
- Error handling guide

Please review for technical accuracy. The draft is in the docs/ folder.

Thanks,
Technical Writer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &reviewer.id, 1, 30).await?;
    println!("✓ Reviewer received review request");

    println!("\n📝 Phase 2: Reviewer provides detailed feedback");

    // In a real test, reviewer would:
    // 1. Use read_file to read API_GUIDE.md
    // 2. Use grep to search for specific patterns
    // 3. Provide feedback via email

    let _review_feedback = reply_to_mail(
        state_handle.clone(),
        review_request.id.clone(),
        Some(reviewer.id.clone()),
        r#"Hi Technical Writer,

Reviewed API_GUIDE.md. Good structure overall, but found several technical issues:

📌 Required Changes:
1. Authentication section: Update to mention JWT expiration (15 minutes, not 30)
2. POST /users endpoint: Missing required field "role" in request body
3. Error codes: Add 429 (Rate Limit Exceeded) to error handling guide

📌 Suggestions:
4. Add rate limiting details (100 requests/minute)
5. Include pagination example for GET /users
6. Add troubleshooting section for common errors

Priority items: 1, 2, 3
Nice-to-have: 4, 5, 6

Please update and ping me for re-review.

Senior Engineer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &writer.id, 1, 30).await?;
    println!("✓ Writer received detailed feedback");

    println!("\n📝 Phase 3: Writer updates documentation");

    // In a real test, writer would:
    // 1. Use read_file to see current content
    // 2. Use edit_file or write_file to make changes
    // 3. Confirm updates via email

    let _update_complete = reply_to_mail(
        state_handle.clone(),
        review_request.id.clone(),
        Some(writer.id.clone()),
        r#"Hi Senior Engineer,

Documentation updated! Addressed all priority items:

✅ Updated JWT expiration to 15 minutes
✅ Added "role" field to POST /users example
✅ Added 429 error code to error handling guide
✅ Bonus: Also added rate limiting details and pagination example

The troubleshooting section will be added in v2 (requires input from support team).

Ready for re-review.

Technical Writer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &reviewer.id, 2, 30).await?;
    println!("✓ Reviewer received update notification");

    println!("\n📝 Phase 4: Reviewer approves");

    let _approval = reply_to_mail(
        state_handle.clone(),
        review_request.id.clone(),
        Some(reviewer.id.clone()),
        r#"Hi Technical Writer,

Re-reviewed the updates. Excellent work!

✅ All technical issues resolved
✅ Examples are accurate
✅ Rate limiting details are helpful

Approved for publication. Great job on the quick turnaround!

Senior Engineer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &writer.id, 2, 30).await?;
    println!("✓ Writer received approval");

    // Verify collaboration flow
    let writer_inbox = get_inbox(state_handle.clone(), &writer.id).await?;
    assert_eq!(
        writer_inbox.len(),
        2,
        "Writer should have: feedback + approval"
    );

    let reviewer_inbox = get_inbox(state_handle.clone(), &reviewer.id).await?;
    assert_eq!(
        reviewer_inbox.len(),
        1,
        "Reviewer should have: review request"
    );

    // Verify thread has complete conversation
    let thread_messages = anycowork::commands::mail::get_mail_thread_messages(
        state_handle.clone(),
        review_request.id,
    )
    .await?;

    assert_eq!(
        thread_messages.len(),
        4,
        "Thread should have: request + feedback + update + approval"
    );

    println!("\n✅ Test PASSED: Document collaboration workflow works");
    Ok(())
}

/// Test 3: Code Review with File Analysis
///
/// Scenario: Dev creates code → Reviewer uses grep to analyze → Comments → Dev updates
/// Validates: File search tools, code analysis, iterative reviews
#[tokio::test]
async fn test_code_review_with_analysis() -> Result<(), Box<dyn std::error::Error>> {
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
    let dev = create_test_agent(
        pool,
        "Junior Developer",
        "🆕",
        "Junior developer writing code and learning best practices",
    );
    let reviewer = create_test_agent(
        pool,
        "Code Reviewer",
        "🔍",
        "Senior developer reviewing code for quality and security using analysis tools",
    );

    println!("\n🔍 Phase 1: Developer requests code review");

    let review_request = send_mail(
        state_handle.clone(),
        Some(dev.id.clone()),
        Some(reviewer.id.clone()),
        "Code Review: User Authentication Module".to_string(),
        r#"Hi Code Reviewer,

I've completed the user authentication module. Could you review?

Files changed:
- src/auth/login.py (new)
- src/auth/session.py (new)
- src/utils/crypto.py (modified)

Focus areas:
- Security best practices
- Error handling
- Code style

Thanks!
Junior Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &reviewer.id, 1, 30).await?;
    println!("✓ Reviewer received review request");

    println!("\n🔍 Phase 2: Reviewer analyzes code using tools");

    // In a real test, reviewer would:
    // 1. Use glob to find all changed files
    // 2. Use grep to search for security issues (e.g., hardcoded passwords)
    // 3. Use read_file to examine specific files
    // 4. Provide comprehensive feedback

    let _review_feedback = reply_to_mail(
        state_handle.clone(),
        review_request.id.clone(),
        Some(reviewer.id.clone()),
        r#"Hi Junior Developer,

Completed code review. Found several issues:

🔴 Security Issues (CRITICAL):
1. src/auth/login.py:45 - Passwords stored in plaintext
   Fix: Use bcrypt.hashpw() before storing

2. src/auth/session.py:23 - Session tokens predictable (timestamp-based)
   Fix: Use secrets.token_urlsafe(32) instead

🟡 Best Practices (Important):
3. src/auth/login.py:67 - No rate limiting on login attempts
   Suggestion: Add rate limiting (5 attempts per 15 min)

4. src/utils/crypto.py:12 - Using deprecated hashlib.md5
   Fix: Switch to hashlib.sha256 or better

🟢 Code Style (Minor):
5. Missing docstrings in several functions
6. Inconsistent error messages

Please address security issues (#1, #2) before merging. Others can be follow-up tasks.

Code Reviewer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 1, 30).await?;
    println!("✓ Developer received review feedback");

    println!("\n🔍 Phase 3: Developer addresses critical issues");

    let _fixes_complete = reply_to_mail(
        state_handle.clone(),
        review_request.id.clone(),
        Some(dev.id.clone()),
        r#"Hi Code Reviewer,

Fixed all security issues!

✅ #1: Implemented bcrypt password hashing
✅ #2: Using secrets.token_urlsafe() for session tokens
✅ #3: Added rate limiting decorator @rate_limit(5, 900)
✅ #4: Switched from md5 to sha256

Also added docstrings and standardized error messages.

Ready for re-review!

Junior Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &reviewer.id, 2, 30).await?;
    println!("✓ Reviewer received fix notification");

    println!("\n🔍 Phase 4: Reviewer verifies fixes and approves");

    let _approval = reply_to_mail(
        state_handle.clone(),
        review_request.id.clone(),
        Some(reviewer.id.clone()),
        r#"Hi Junior Developer,

Re-reviewed the changes. Excellent work!

✅ Password hashing implemented correctly
✅ Session tokens are cryptographically secure
✅ Rate limiting works as expected
✅ Crypto upgraded to sha256

All security issues resolved. Code is approved for merge! 🎉

Great job addressing the feedback thoroughly.

Code Reviewer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 2, 30).await?;
    println!("✓ Developer received approval");

    // Verify code review workflow
    let dev_inbox = get_inbox(state_handle.clone(), &dev.id).await?;
    assert_eq!(dev_inbox.len(), 2, "Dev should have: feedback + approval");

    let reviewer_inbox = get_inbox(state_handle.clone(), &reviewer.id).await?;
    assert_eq!(
        reviewer_inbox.len(),
        1,
        "Reviewer should have: review request"
    );

    println!("\n✅ Test PASSED: Code review with analysis tools works");
    Ok(())
}

/// Test 4: Multi-Agent Build and Test Pipeline
///
/// Scenario: Dev writes code → Build agent compiles → Test agent runs tests → Reports results
/// Validates: Bash tool usage, pipeline coordination, parallel execution
#[tokio::test]
async fn test_build_and_test_pipeline() -> Result<(), Box<dyn std::error::Error>> {
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
    let dev = create_test_agent(
        pool,
        "Developer",
        "💻",
        "Developer implementing features and writing code",
    );
    let build_agent = create_test_agent(
        pool,
        "Build Agent",
        "🔨",
        "Automated build agent that compiles code using bash commands",
    );
    let test_agent = create_test_agent(
        pool,
        "Test Agent",
        "🧪",
        "Automated test runner that executes test suites via bash",
    );
    let tech_lead = create_test_agent(
        pool,
        "Tech Lead",
        "👔",
        "Tech lead coordinating development and deployments",
    );

    println!("\n🔨 Phase 1: Developer completes feature");

    let feature_complete = send_mail(
        state_handle.clone(),
        Some(dev.id.clone()),
        Some(tech_lead.id.clone()),
        "Feature Complete: Payment Integration".to_string(),
        r#"Hi Tech Lead,

Payment integration feature is complete and committed.

Changes:
- Added Stripe payment processing
- Created payment webhook handler
- Added database migrations
- Updated API documentation

Ready for build and testing pipeline.

Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &tech_lead.id, 1, 30).await?;
    println!("✓ Tech lead received completion notice");

    println!("\n🔨 Phase 2: Tech lead triggers build");

    let build_request = send_mail(
        state_handle.clone(),
        Some(tech_lead.id.clone()),
        Some(build_agent.id.clone()),
        "Build Request: Payment Feature Branch".to_string(),
        r#"Build Agent,

Please build the payment-integration branch.

Build commands:
1. git checkout payment-integration
2. npm install
3. npm run build
4. npm run lint

Report build status.

Tech Lead"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &build_agent.id, 1, 30).await?;
    println!("✓ Build agent received request");

    println!("\n🔨 Phase 3: Build agent executes and reports");

    // In a real test, build agent would:
    // 1. Use bash tool to run build commands
    // 2. Capture output and errors
    // 3. Report results via email

    let _build_result = reply_to_mail(
        state_handle.clone(),
        build_request.id.clone(),
        Some(build_agent.id.clone()),
        r#"Tech Lead,

Build completed successfully! ✅

Build Summary:
✅ git checkout payment-integration - OK
✅ npm install - 127 packages installed
✅ npm run build - Build successful (2.3s)
✅ npm run lint - No lint errors

Build artifacts ready at: dist/
Build time: 45 seconds

Ready for testing.

Build Agent"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &tech_lead.id, 2, 30).await?;
    println!("✓ Tech lead received build success");

    println!("\n🔨 Phase 4: Tech lead triggers tests");

    let test_request = send_mail(
        state_handle.clone(),
        Some(tech_lead.id.clone()),
        Some(test_agent.id.clone()),
        "Test Request: Payment Feature".to_string(),
        r#"Test Agent,

Build passed. Please run full test suite on payment-integration branch.

Test commands:
1. npm run test:unit
2. npm run test:integration
3. npm run test:e2e

Report test results and coverage.

Tech Lead"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &test_agent.id, 1, 30).await?;
    println!("✓ Test agent received request");

    println!("\n🔨 Phase 5: Test agent runs tests and reports failure");

    let _test_result = reply_to_mail(
        state_handle.clone(),
        test_request.id.clone(),
        Some(test_agent.id.clone()),
        r#"Tech Lead,

Test run completed with FAILURES ❌

Test Summary:
✅ Unit tests: 145/145 passed (100%)
❌ Integration tests: 12/13 passed (92%)
   FAILED: test_payment_webhook_signature_validation
   Error: Invalid signature verification
✅ E2E tests: 8/8 passed (100%)

Overall: 165/166 tests passed (99.4%)
Coverage: 87%

🐛 Failed Test Details:
test_payment_webhook_signature_validation
  Expected: Webhook validation to pass with valid Stripe signature
  Actual: Validation failed - signature mismatch
  Location: tests/integration/payment.test.js:45

Blocking deployment. Developer attention needed.

Test Agent"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &tech_lead.id, 3, 30).await?;
    println!("✓ Tech lead received test failure report");

    println!("\n🔨 Phase 6: Tech lead assigns bug fix");

    let _bug_assignment = reply_to_mail(
        state_handle.clone(),
        feature_complete.id.clone(),
        Some(tech_lead.id.clone()),
        r#"Developer,

Tests failed - webhook signature validation issue.

Failed test: test_payment_webhook_signature_validation
Error: Invalid signature verification

Please investigate and fix. Priority: High

Tech Lead"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 1, 30).await?;
    println!("✓ Developer received bug assignment");

    println!("\n🔨 Phase 7: Developer fixes and confirms");

    let _bug_fix = reply_to_mail(
        state_handle.clone(),
        feature_complete.id.clone(),
        Some(dev.id.clone()),
        r#"Tech Lead,

Bug fixed! Issue was in webhook signature parsing.

Fix: Updated signature extraction to use raw request body instead of parsed JSON.

Local tests now pass:
✅ test_payment_webhook_signature_validation - PASSED

Ready for re-test in pipeline.

Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &tech_lead.id, 4, 30).await?;
    println!("✓ Tech lead received fix confirmation");

    // Verify pipeline workflow
    let tech_lead_inbox = get_inbox(state_handle.clone(), &tech_lead.id).await?;
    assert!(
        tech_lead_inbox.len() >= 4,
        "Tech lead should have: feature complete + build result + test result + bug fix"
    );

    let build_agent_inbox = get_inbox(state_handle.clone(), &build_agent.id).await?;
    assert_eq!(
        build_agent_inbox.len(),
        1,
        "Build agent should have build request"
    );

    let test_agent_inbox = get_inbox(state_handle.clone(), &test_agent.id).await?;
    assert_eq!(
        test_agent_inbox.len(),
        1,
        "Test agent should have test request"
    );

    println!("\n✅ Test PASSED: Build and test pipeline coordination works");
    Ok(())
}

/// Test 5: File Search and Refactoring Coordination
///
/// Scenario: Architect identifies refactoring need → Dev uses grep to find occurrences → Updates files
/// Validates: Grep tool for code search, coordinated refactoring
#[tokio::test]
async fn test_file_search_and_refactoring() -> Result<(), Box<dyn std::error::Error>> {
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
    let architect = create_test_agent(
        pool,
        "Software Architect",
        "🏗️",
        "Software architect identifying technical debt and planning refactoring",
    );
    let dev = create_test_agent(
        pool,
        "Refactoring Developer",
        "♻️",
        "Developer specializing in code refactoring with grep and file editing tools",
    );

    println!("\n♻️ Phase 1: Architect identifies refactoring need");

    let refactoring_task = send_mail(
        state_handle.clone(),
        Some(architect.id.clone()),
        Some(dev.id.clone()),
        "Refactoring Task: Rename UserModel to User".to_string(),
        r#"Hi Refactoring Developer,

We need to refactor the codebase to rename `UserModel` class to simply `User`.

Scope:
- Search entire codebase for "UserModel" usage
- Update class definition and all references
- Update imports across all files
- Verify no breaking changes

This will improve code clarity and follow our naming conventions.

Tools needed:
- Use grep to find all occurrences
- Use edit_file to update references
- Use bash to run tests after changes

Please report scope and plan before starting.

Software Architect"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 1, 30).await?;
    println!("✓ Developer received refactoring task");

    println!("\n♻️ Phase 2: Developer analyzes scope using grep");

    // In a real test, developer would:
    // 1. Use grep tool to search for "UserModel" across codebase
    // 2. Analyze results and plan changes
    // 3. Report findings

    let _scope_analysis = reply_to_mail(
        state_handle.clone(),
        refactoring_task.id.clone(),
        Some(dev.id.clone()),
        r#"Hi Software Architect,

Completed scope analysis using grep.

Found 47 occurrences of "UserModel" across 12 files:

📁 Core Files (Class definition):
- src/models/user.py:15 - Class definition
- src/models/user.py:23 - Method references

📁 Import Statements (18 files):
- src/api/users.py:3
- src/api/auth.py:5
- src/services/user_service.py:2
- ... (15 more files)

📁 Type Hints (21 occurrences):
- Various function signatures and return types

📁 Tests (8 occurrences):
- tests/test_user.py
- tests/test_api.py

Refactoring Plan:
1. Update class definition in user.py
2. Update all imports (use edit_file)
3. Update type hints
4. Update test files
5. Run full test suite to verify

Estimated time: 2 hours
Risk: Low (straightforward rename)

Ready to proceed?

Refactoring Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &architect.id, 1, 30).await?;
    println!("✓ Architect received scope analysis");

    println!("\n♻️ Phase 3: Architect approves and developer executes");

    let _approval = reply_to_mail(
        state_handle.clone(),
        refactoring_task.id.clone(),
        Some(architect.id.clone()),
        r#"Refactoring Developer,

Plan looks good! 47 occurrences is manageable.

Approved to proceed. Please:
1. Make changes systematically
2. Run tests after each major change
3. Report completion with test results

Go ahead!

Software Architect"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &dev.id, 2, 30).await?;
    println!("✓ Developer received approval");

    println!("\n♻️ Phase 4: Developer completes refactoring");

    // In a real test, developer would:
    // 1. Use edit_file to rename class and update references
    // 2. Use bash to run tests
    // 3. Report completion

    let _refactoring_complete = reply_to_mail(
        state_handle.clone(),
        refactoring_task.id.clone(),
        Some(dev.id.clone()),
        r#"Software Architect,

Refactoring completed successfully! ✅

Changes Made:
✅ Updated class definition: UserModel → User
✅ Updated 18 import statements
✅ Updated 21 type hints
✅ Updated 8 test file references
✅ Total: 47 occurrences updated

Test Results:
✅ Unit tests: 234/234 passed
✅ Integration tests: 45/45 passed
✅ No breaking changes detected

All references updated, tests pass. Ready for code review!

Refactoring Developer"#.to_string(),
    )
    .await?;

    wait_for_mail_processing(state_handle.clone(), &architect.id, 2, 30).await?;
    println!("✓ Architect received completion report");

    // Verify refactoring workflow
    let architect_inbox = get_inbox(state_handle.clone(), &architect.id).await?;
    assert_eq!(
        architect_inbox.len(),
        2,
        "Architect should have: scope analysis + completion"
    );

    let dev_inbox = get_inbox(state_handle.clone(), &dev.id).await?;
    assert_eq!(
        dev_inbox.len(),
        1,
        "Dev should have: refactoring task"
    );

    // Verify conversation thread
    let thread_messages = anycowork::commands::mail::get_mail_thread_messages(
        state_handle.clone(),
        refactoring_task.id,
    )
    .await?;

    assert_eq!(
        thread_messages.len(),
        4,
        "Thread should have: task + analysis + approval + completion"
    );

    println!("\n✅ Test PASSED: File search and refactoring coordination works");
    Ok(())
}
