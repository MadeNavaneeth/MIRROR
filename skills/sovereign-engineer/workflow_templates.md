# Sovereign Engineer Workflow Templates

These templates define the exact chains of thought and tool sequences for the Sovereign Engineer.

## Template: The Dev-to-PR Cycle

**Trigger**: New Issue/Task detected via Ingress.

### Phase 1: Contextualization
1. **Memory Recall**: Search SQLite/Vector memory for similar past issues or relevant code modules.
   - *Tool*: `memory_recall`
2. **Code Exploration**: Grep the codebase for symbols mentioned in the task.
   - *Tool*: `shell("grep -r ...")`
3. **Reproduction (Web/UI)**: If the issue is web-related, use Playwright to attempt a reproduction.
   - *Tool*: `playwright_mcp`

### Phase 2: Implementation
1. **Branching**: Create a fresh feature or fix branch.
   - *Tool*: `shell("git checkout -b ...")`
2. **Drafting**: Use the File tools to apply changes.
   - *Tool*: `file_write` / `multi_replace_file_content`
3. **Validation**: Run the project's test suite.
   - *Tool*: `shell("cargo test" or "npm test")`
4. **Refinement**: Iterate until all tests pass.

### Phase 3: Submission & Reporting
1. **PR Creation**: Open a Pull Request on GitHub.
   - *Tool*: `github.create_pull_request`
2. **Documentation**: Append the session summary to the daily engineering log in Obsidian.
   - *Tool*: `obsidian_direct.append_note`
3. **Voice Briefing**: (Optional) Generate a short audio summary of the fix.
   - *Tool*: `elevenlabs_agents.speak`

---

## Template: Daily Intelligence Synthesis

**Trigger**: End of day (scheduled) or manual request.

1. **Log Aggregation**: Read today's notes from Obsidian.
2. **Memory Promotion**: Identify key architecture decisions or bug fixes and store them in long-term memory.
   - *Tool*: `memory_store`
3. **Email Digest**: Send a structured summary of the day's achievements.
   - *Tool*: `agentmail.send`
