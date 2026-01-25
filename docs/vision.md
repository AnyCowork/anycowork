# AnyCowork Vision: The Smart, Safe, & Optimized AI Coworker

**Goal**: To build the ultimate open-source AI coworker that lives on your device, respects your privacy, and integrates with your entire workflow.

## 1. Smart: Capabilities & Extensibility
AnyCowork isn't just a chatbot; it's a proactive coworker.
- **SOTA Models**: First-class support for Gemini 3 Pro, Claude 3.5 Sonnet, and GPT-4o.
- **MCP Native**: Implements the [Model Context Protocol](https://modelcontextprotocol.io) to connect to any external data source (GitHub, Google Drive, Postgres) without custom code.
- **Skills System**: Extensible "Skills" (directories with `SKILL.md` + scripts) allow users to teach the agent new workflows.
- **Agentic Architecture**: Uses a **Coordinator-Worker** pattern. The Coordinator plans complex tasks, and Workers execute them in parallel (future) or sequence.

## 2. Safe: Safety by Design
We prioritize user control and system integrity.
- **Human-in-the-Loop**: Critical actions (file edits, shell commands) always require explicit user confirmation.
- **Granular Permissions**: "Always allow read-only", "Ask for write". Users define the boundaries.
- **Local-First Privacy**: Your data stays on your machine. No hidden cloud sync.
- **Guardrails**: System prompts include strict safety guidelines to prevent accidental data loss.

## 3. Optimized: Performance & Efficiency
Built for the edge, not the server farm.
- **Powered by Rust & Tauri**: Extremely small binary size (<20MB) and minimal RAM usage compared to Electron apps.
- **Fast Mode**: Instant responses for simple queries, bypassing the heavy planning agent.
- **Context Optimization**: Smart token management ensures long-running tasks don't blow up context windows or costs.
- **Robust Flow**: The event-driven architecture ensures UI responsiveness even during heavy agent tasks.

## 4. The "Coworker" Experience
- **Delegation**: "Here's a folder, fix the docs." -> Agent plans -> execs -> reports back.
- **Collaboration**: Multiple specialist agents (Coder, Writer, Researcher) working together on the same task.
