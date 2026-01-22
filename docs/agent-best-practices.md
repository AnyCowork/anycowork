# Agent Best Practices for Daily Workflows

> **Last Updated:** 2026-01-22
> **Version:** 1.0.0

This document provides comprehensive guidelines for optimizing agent performance and interaction patterns for daily use.

## Table of Contents

1. [Overview](#overview)
2. [Agent Configuration](#agent-configuration)
3. [Interaction Patterns](#interaction-patterns)
4. [Performance Optimization](#performance-optimization)
5. [Testing Guidelines](#testing-guidelines)
6. [Common Scenarios](#common-scenarios)

---

## Overview

AnyCowork agents are designed to be efficient, natural, and context-aware assistants for daily workflows. This guide ensures your agents provide the best user experience while maintaining high performance.

### Core Principles

1. **Efficiency First**: Minimize unnecessary back-and-forth
2. **Natural Interaction**: Conversational, not robotic
3. **Context Awareness**: Remember conversation history
4. **Safety**: Confirm destructive operations
5. **Proactive Help**: Anticipate needs without being intrusive

---

## Agent Configuration

### Personality Types

Choose the right personality for your workflow:

#### 1. Concise Agent
**Use for:** Quick tasks, file operations, simple queries

```rust
personality: "concise"
tone: "direct"
system_prompt: "You are a concise assistant. Provide brief, direct answers.
                Only ask questions when absolutely necessary."
```

**Characteristics:**
- Minimal interaction (1 turn when possible)
- Direct responses
- Acts on clear instructions immediately

#### 2. Interactive Agent
**Use for:** Complex tasks, ambiguous requests, collaborative work

```rust
personality: "interactive"
tone: "friendly"
system_prompt: "You are an interactive assistant. Engage with users,
                ask clarifying questions when needed, but don't over-communicate."
```

**Characteristics:**
- Asks clarifying questions for ambiguous requests
- Provides context and explanations
- Balances efficiency with thoroughness

#### 3. Proactive Agent
**Use for:** Code reviews, analysis, multi-step workflows

```rust
personality: "proactive"
tone: "professional"
system_prompt: "You are a proactive assistant. Anticipate user needs
                and offer helpful suggestions, but respect user autonomy."
```

**Characteristics:**
- Anticipates next steps
- Offers suggestions
- Handles multi-step tasks autonomously

### AI Model Selection

| Model | Best For | Temperature | Max Tokens |
|-------|----------|-------------|------------|
| `gpt-4o` | General tasks, balanced | 0.7 | 4096 |
| `gpt-4o-mini` | Simple tasks, fast responses | 0.5 | 2048 |
| `claude-opus-4.5` | Complex reasoning, code | 0.7 | 8192 |
| `gemini-2.0-flash` | Fast, cost-effective | 0.6 | 4096 |

---

## Interaction Patterns

### Optimal Turn Counts

Follow these guidelines for interaction efficiency:

| Request Type | Expected Turns | Example |
|--------------|----------------|---------|
| **Simple Read** | 1 | "Show me config.json" |
| **Simple Write** | 1 | "Create README.md" |
| **Ambiguous** | 2 | "Search for errors" → "In which directory?" |
| **Destructive** | 2 | "Delete logs" → "Found 5 files. Confirm?" |
| **Complex Multi-Step** | 1 | "Review recent changes" (agent handles autonomously) |
| **Vague** | 2 | "Fix it" → "What should I fix?" |

### When to Ask Questions

✅ **DO Ask When:**
- User request is genuinely ambiguous ("search for TODO" - where?)
- Destructive operation requires confirmation (delete, overwrite)
- User input is too vague to proceed ("fix it" - fix what?)
- Multiple valid interpretations exist

❌ **DON'T Ask When:**
- Context provides the answer (previous messages)
- Reasonable default exists (current directory for "list files")
- User provided all necessary information
- Question would be redundant

### Anti-Patterns to Avoid

#### 1. Over-Confirmation
```
❌ BAD:
User: "Create a file test.txt"
Agent: "Should I create the file in the current directory?"
Agent: "What content should I put in it?"
Agent: "Should I overwrite if it exists?"

✅ GOOD:
User: "Create a file test.txt"
Agent: "Created test.txt (empty file)"
```

#### 2. Excessive Explanation
```
❌ BAD:
"I will now execute the filesystem tool with operation='list_dir'
and path='.' to retrieve the directory listing..."

✅ GOOD:
"Here are the files in the current directory:"
```

#### 3. Context Ignorance
```
❌ BAD:
User: "List Python files"
Agent: [Shows list]
User: "Now count the lines in those files"
Agent: "Which files do you mean?"

✅ GOOD:
User: "List Python files"
Agent: [Shows list of 5 .py files]
User: "Now count the lines in those files"
Agent: [Counts lines in the 5 Python files from previous turn]
```

---

## Performance Optimization

### 1. History Management

```rust
use anycowork::agents::optimizations::{trim_history, optimize_history_by_tokens};

// Trim to last 100 messages
trim_history(&mut agent.history, 100);

// Optimize by token count (keep under 4000 tokens)
optimize_history_by_tokens(&mut agent.history, 4000);
```

### 2. Tool Result Caching

Enable caching for frequently used tools:

```rust
use anycowork::agents::optimizations::{ToolResultCache, AgentOptimizationConfig};
use std::time::Duration;

// Create cache with 5-minute TTL
let cache = ToolResultCache::new(Duration::from_secs(300));

// Use in tool execution
let result = execute_tool_with_optimization(
    "filesystem",
    &args,
    || tool.execute(args, ctx),
    &config,
    Some(&cache),
).await?;
```

**Cache Benefits:**
- Reduces redundant filesystem reads
- Faster repeated searches
- Lower API costs

### 3. Timeout Configuration

Set appropriate timeouts for different tools:

```rust
let config = AgentOptimizationConfig {
    tool_timeout: Duration::from_secs(30),  // Default
    max_retries: 3,
    enable_caching: true,
    max_history_size: 100,
    enable_parallel_tools: false,  // Enable for independent operations
};
```

### 4. Parallel Tool Execution

For independent operations (e.g., checking multiple files):

```rust
// Enable in config
config.enable_parallel_tools = true;

// Tools that can run in parallel:
// - Multiple file reads
// - Multiple search operations
// - Independent bash commands
```

---

## Testing Guidelines

### Unit Tests

Test individual agent components:

```bash
cd src-tauri
cargo test --lib agents::tests
```

### Integration Tests

Test realistic workflows:

```bash
cargo test --lib agents::workflow_tests
```

### Performance Benchmarks

```bash
cargo test --lib agents::tests::performance_tests -- --nocapture
```

**Expected Performance:**
- Agent creation: < 1ms per agent
- Agent loop initialization: < 10ms
- Tool execution (cached): < 5ms
- Tool execution (uncached): < 500ms

### Workflow Test Scenarios

Ensure your agents handle these scenarios correctly:

1. **Simple File Listing** - 1 turn, no questions
2. **Ambiguous Search** - 2 turns, one clarification
3. **File Creation** - 1 turn if content provided
4. **Destructive Operation** - 2 turns with confirmation
5. **Multi-Step Workflow** - 1 turn, autonomous handling
6. **Vague Request** - 2 turns, necessary clarification
7. **Batch Operation** - 1 turn, no per-item confirmation
8. **Context-Aware Follow-up** - Uses conversation history

---

## Common Scenarios

### Scenario 1: File Management

**User:** "List files in src/"

**Agent Response (Concise):**
```
Here are the files in src/:
📁 components/
📁 layouts/
📄 App.tsx
📄 main.tsx
📄 globals.css
```

**Interaction:** 1 turn
**Tools used:** `filesystem` (list_dir)

---

### Scenario 2: Code Search

**User:** "Find all TODO comments"

**Agent Response (Interactive):**
```
I found 8 TODO comments:

src/components/Header.tsx:42
  // TODO: Add user profile dropdown

src/routes/ChatPage.tsx:156
  // TODO: Implement message search

[... more results]
```

**Interaction:** 1 turn (assumes current directory)
**Tools used:** `search_files` (grep for "TODO")

---

### Scenario 3: Destructive Operation

**User:** "Delete all .log files"

**Agent Response (Proactive):**
```
I found 12 .log files in the current directory:
- debug.log (2.3 MB)
- error.log (156 KB)
- access.log (8.7 MB)
[... more files]

Total size: 11.2 MB

⚠️ Delete all 12 files? This cannot be undone.
```

**Interaction:** 2 turns (requires confirmation)
**Tools used:** `search_files` (find *.log), then `bash` (rm) after approval

---

### Scenario 4: Multi-Step Analysis

**User:** "Analyze recent code changes"

**Agent Response (Proactive):**
```
Analyzed recent changes (last 5 commits):

📊 Summary:
- 8 files changed
- +247 lines added
- -89 lines removed

🎯 Key Changes:
1. Added chat message caching (ChatPage.tsx)
2. Optimized agent loop initialization (agents/mod.rs)
3. Fixed timestamp comparison in tests (agents/tests.rs)

✅ Code Quality:
- All tests passing
- No security issues detected
- Performance improved by ~15%

💡 Suggestions:
- Consider adding error handling in ChatPage.tsx:234
- Document the new caching strategy
```

**Interaction:** 1 turn (agent handles multiple steps autonomously)
**Tools used:** `bash` (git diff, git log), `search_files` (code analysis)

---

## Monitoring and Metrics

### Key Metrics to Track

1. **Average Turns per Task** - Target: 1.5 or less
2. **User Satisfaction** - Measure via feedback
3. **Tool Execution Time** - Target: < 500ms p95
4. **Cache Hit Rate** - Target: > 40% for repeated operations
5. **Error Rate** - Target: < 1%

### Performance Monitoring

```rust
// Log execution metrics
log::info!("Tool execution: {} completed in {:?}",
    tool_name,
    execution_result.duration
);

// Cache statistics
let stats = cache.stats();
log::info!("Cache: {}/{} entries", stats.size, stats.capacity);
```

---

## Troubleshooting

### Issue: Agent Asks Too Many Questions

**Solution:**
1. Review system prompt - ensure it emphasizes efficiency
2. Check personality setting - use "concise" for simple tasks
3. Provide better context in user queries
4. Update agent to use reasonable defaults

### Issue: Agent Doesn't Ask Necessary Questions

**Solution:**
1. Review personality - switch to "interactive" if too aggressive
2. Check safety settings for destructive operations
3. Ensure agent validates ambiguous requests

### Issue: Slow Performance

**Solution:**
1. Enable caching: `config.enable_caching = true`
2. Reduce history size: `trim_history(&mut history, 50)`
3. Check tool timeouts
4. Monitor tool execution times

### Issue: Context Not Retained

**Solution:**
1. Verify history is being maintained correctly
2. Check `max_history_size` isn't too small
3. Ensure session management is working
4. Review message persistence to database

---

## Conclusion

Following these best practices ensures your agents provide:

✅ **Efficient** - Minimal interaction overhead
✅ **Natural** - Conversational and context-aware
✅ **Reliable** - Safe handling of operations
✅ **Fast** - Optimized performance
✅ **Scalable** - Ready for daily production use

For questions or improvements, please contribute to the [AnyCowork repository](https://github.com/vietanhdev/anycowork).

---

## Quick Reference Card

```
INTERACTION LEVELS:
├─ Simple Operations: 1 turn
├─ Ambiguous Requests: 2 turns (1 clarification)
├─ Destructive Actions: 2 turns (1 confirmation)
└─ Complex Workflows: 1 turn (autonomous)

PERSONALITY GUIDE:
├─ Concise → Fast, direct, minimal interaction
├─ Interactive → Balanced, asks when needed
└─ Proactive → Anticipates, suggests, autonomou

OPTIMIZATION:
├─ Enable caching for repeated operations
├─ Trim history to last 100 messages
├─ Set timeouts: 30s default
└─ Monitor performance metrics

TESTING:
├─ Unit tests: cargo test agents::tests
├─ Workflow tests: cargo test agents::workflow_tests
└─ Performance: cargo test --nocapture | grep "in"
```

---

**Built with ❤️ for efficient AI assistance**
