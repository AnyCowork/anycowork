# End-to-End Test Suite

This directory contains comprehensive end-to-end tests for the AnyCowork application.

## Test Files

### `e2e_mail.rs`
Tests the complete mail system workflow including:
- Sending emails between characters
- Email thread management
- Message filtering (inbox vs sent)
- User-to-agent and agent-to-user emails
- Special character handling
- Name resolution

### `e2e_tool_calling.rs`
Tests tool calling infrastructure including:
- JSON parsing with various formats (pure JSON, embedded, markdown)
- Escape sequence handling
- Multiple tool calls in one response
- Nested JSON arguments
- Invalid JSON handling
- Message deduplication logic

### `e2e_agent_mail_integration.rs`
Integration tests for agent-to-agent communication:
- Tool execution (SendEmailTool)
- Recipient name resolution (partial matching)
- Email body formatting preservation
- Tool validation (missing/unknown recipients)
- Send to user mailbox

### `e2e_transcribe.rs`
Tests the transcription feature:
- Model status checking
- Sample audio transcription
- Audio file validation

### `e2e_office_workflows_basic.rs` ⭐ NEW
Phase 1: Foundation office workflow tests:
- **Test 1.1**: Simple agent-to-agent messaging with replies
- **Test 1.2**: Group communication (3+ agents) with multiple replies
- **Test 3.1**: Task assignment and progress tracking via email threads

### `e2e_office_workflows_coordination.rs` ⭐ NEW
Phase 2: Coordination workflow tests:
- **Test 2.1**: Meeting scheduling with negotiation and rescheduling
- **Test 3.2**: Escalation workflow (developer → PM → tech lead → resolution)
- **Test 4.1**: Research task distribution and parallel synthesis

### `e2e_office_workflows_complex.rs` ⭐ NEW
Phase 3: Complex multi-agent workflow tests:
- **Test 2.2**: Cross-team meeting coordination (6 agents across 2 teams)
- **Test 3.3**: Parallel task coordination with bug tracking and convergence
- **Test 7.1**: Feature launch coordination (cross-functional departments)

### `e2e_office_workflows_advanced.rs` ⭐ NEW
Phase 4: Advanced workflow patterns and edge cases:
- **Test 1.3**: Email forwarding pattern (information flow across parties)
- **Test 4.2**: Multi-round clarification loops
- **Test 5.1**: Document review workflow (collaborative feedback)
- **Test 6.1**: Consensus building (multi-party decision making)

### `e2e_office_workflows_tools.rs` ⭐ NEW
Phase 5: Tool-based collaboration workflows:
- **Test 1**: Code Implementation Workflow (PM → Dev writes code → QA tests → Bug fix cycle)
- **Test 2**: Document Collaboration (Writer creates → Reviewer reads/suggests → Writer updates)
- **Test 3**: Code Review with Analysis (Dev submits → Reviewer uses grep → Comments → Dev fixes)
- **Test 4**: Build and Test Pipeline (Dev commits → Build agent → Test agent → Failure reporting)
- **Test 5**: File Search and Refactoring (Architect plans → Dev uses grep → Systematic updates)

**Tools Validated**: File operations (read/write/edit), bash commands, grep/glob for code search, coordinated tool usage

### `e2e_agent_tool_execution.rs` ⭐ NEW
Actual agent tool execution tests (agents invoke real tools):
- **Test 1**: File Creation - Agent creates file using filesystem tool
- **Test 2**: File Reading - Agent reads existing file
- **Test 3**: File Search - Agent searches files using search_files/grep
- **Test 4**: Bash Execution - Agent executes bash commands
- **Test 5**: Multi-step Workflow - Agent creates file then reads it back

**Real Tool Execution**: Unlike simulation tests, these invoke the Coordinator with agents in autonomous mode, causing actual tool calls to be executed and verified through event capture.

## Running Tests

### Run all e2e tests
```bash
cd src-tauri
cargo test --tests
```

### Run specific test file
```bash
cargo test --test e2e_mail
cargo test --test e2e_tool_calling
cargo test --test e2e_agent_mail_integration
cargo test --test e2e_office_workflows_basic
cargo test --test e2e_office_workflows_coordination
cargo test --test e2e_office_workflows_complex
cargo test --test e2e_office_workflows_advanced
cargo test --test e2e_office_workflows_tools
```

### Run all office workflow tests
```bash
cargo test e2e_office_workflows
```

### Run tool-based collaboration tests specifically
```bash
cargo test --test e2e_office_workflows_tools
```

### Run actual tool execution tests
```bash
# Requires OPENAI_API_KEY environment variable
cargo test --test e2e_agent_tool_execution -- --nocapture
```

### Run specific test
```bash
cargo test test_send_email_between_characters
```

### Run with output
```bash
cargo test -- --nocapture
```

## Test Coverage

### Message Filtering
- ✅ Duplicate message prevention
- ✅ Role-based message filtering
- ✅ Recent message deduplication (last 5 messages)

### Tool Calling
- ✅ Pure JSON parsing
- ✅ Embedded JSON in text
- ✅ Markdown code blocks
- ✅ Multiple tool calls
- ✅ Escape sequence handling
- ✅ Special characters and Unicode
- ✅ Nested JSON arguments

### Mail Handling
- ✅ Agent-to-agent emails
- ✅ User-to-agent emails
- ✅ Agent-to-user emails
- ✅ Thread creation and management
- ✅ Reply generation without tool calls
- ✅ Name resolution (exact and partial)
- ✅ Inbox vs sent filtering
- ✅ Message formatting preservation

### Office Workflow Integration (NEW)
- ✅ **Basic Patterns**: Simple messaging, group communication, task assignments
- ✅ **Coordination**: Meeting scheduling, escalation chains, parallel research
- ✅ **Complex Workflows**: Cross-team coordination (6+ agents), parallel convergence, feature launches
- ✅ **Advanced Patterns**: Email forwarding, clarification loops, document reviews, consensus building
- ✅ **Background Processing**: Automatic agent mail responses
- ✅ **Thread Continuity**: Multi-message conversations maintain context
- ✅ **Folder Management**: Inbox/sent folder separation per agent

**Coverage**: 13 comprehensive test cases validating real-world office collaboration scenarios

### Tool-Based Collaboration (NEW)
- ✅ **Code Implementation**: Full dev cycle with file creation, testing, and bug fixes
- ✅ **Document Collaboration**: Multi-agent document editing with feedback loops
- ✅ **Code Review**: File analysis using grep, security reviews, iterative fixes
- ✅ **Build Pipelines**: Automated build and test agents with bash commands
- ✅ **Refactoring**: Grep-based code search, systematic file updates, test verification
- ✅ **Multi-Tool Workflows**: Combined use of read_file, write_file, edit_file, bash, grep, glob
- ✅ **Quality Assurance**: Testing workflows with failure detection and bug reporting

**Coverage**: 5 comprehensive test cases validating realistic development workflows with tool usage

### Actual Tool Execution (NEW)
- ✅ **File Creation**: Agent creates files with filesystem tool
- ✅ **File Reading**: Agent reads existing files
- ✅ **File Search**: Agent searches content using grep/search_files
- ✅ **Bash Execution**: Agent runs bash commands
- ✅ **Multi-step Workflows**: Agent chains multiple tool calls together
- ✅ **Event Capture**: TestObserver captures and verifies tool execution
- ✅ **Autonomous Mode**: Auto-approval of tool permissions for testing

**Coverage**: 5 comprehensive test cases validating actual agent tool execution with real API calls

**Total Test Suite**: 23 test cases (13 communication + 5 tool simulation + 5 actual execution)

## Known Issues

### Windows DirectML
Some test binaries may fail to start on Windows due to DirectML DLL issues (STATUS_ENTRYPOINT_NOT_FOUND). This is a pre-existing environment issue affecting integration test binaries that link against the ort crate.

Workaround: Run tests in the anyagents crate directly:
```bash
cd anyagents
cargo test
```

## Test Data

All tests use in-memory SQLite databases (`create_test_pool()`) and clean up after themselves. No persistent state is created during testing.

## Debugging Tests

### Enable debug logging
```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Run single test with backtrace
```bash
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

## CI/CD Integration

These tests are designed to run in CI environments where proper DLL configuration is available. They serve as both runtime tests and compile-time verification.
