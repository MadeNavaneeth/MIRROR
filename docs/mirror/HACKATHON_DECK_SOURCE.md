# Mirror: The Sovereign Personal Intelligence System
> **Presentation Source Document for Samsung PRISM — Clash of the Claws**

---

## 1. Project Identity
- **Name**: Mirror
- **Tagline**: The AI that doesn't just talk — it acts to align your life.
- **Theme**: Daily Utility & Personal Productivity (Autonomous Multi-Agent Runtime).

---

## 2. The Core Problem: "The Intention-Action Gap"
- **The Gap**: We plan our day in the morning (Intention), but by evening, we've drifted due to distractions or lack of accountability (Action).
- **The Fragmentation**: Current tools are passive. Calendars don't know why you missed a meeting; journals don't tell you how to fix your schedule.
- **The Result**: Slow behavioral drift away from career and health goals.

---

## 3. The Solution: Mirror
Mirror is an **Autonomous Personal Intelligence System** that continuously monitors, plans, and audits your life to ensure your daily actions match your long-term ambitions.

### Key Innovation: The "Closed-Loop" Autonomy
1. **Observe**: Journal Agent captures reality.
2. **Analyze**: Health & Career agents detect drift.
3. **Act**: Planner Agent recalibrates the schedule proactively.

---

## 4. Multi-Agent Architecture (The 4 Pillars)
Mirror utilizes 4 strictly specialized agents to ensure precision and zero overlap:

### 📓 1. Journal Agent (The Input Engine)
- **Role**: Captures speech and text inputs.
- **Task**: Classifies entries into Informational, Behavioral (Drift), or Critical (Plan-breaking).
- **Output**: Updates the "User State Graph" in local memory.

### 📅 2. Planner Agent (The Time Architect)
- **Role**: Controls the daily timeline.
- **Task**: Dynamically allocates "Skill Blocks" and "Health Blocks."
- **Autonomy**: If a task is missed, it automatically reschedules or flags a "Recovery Day."

### 🚀 3. Career Agent (The Strategist)
- **Role**: Long-term alignment auditor.
- **Task**: Maps daily proof-of-work against target job descriptions and skill gaps.
- **Feedback**: Alerts the user when their daily behavior doesn't support their "Target Role."

### 🧘 4. Health Agent (The Consistency Auditor)
- **Role**: Behavioral foundation tracker.
- **Task**: Monitors sleep, focus patterns, and consistency streaks.
- **Metric**: Generates the "Consistency Score" (0-100) to measure system health.

---

## 5. Technical Excellence (The "Under the Hood")

### Performance & Sovereignty
- **Runtime**: High-performance Rust backend (Mirror) for low-latency agentic loops.
- **Sovereignty**: 100% Local SQLite + Vector Memory. Data never leaves the user's machine.
- **Proactive Engine**: A "Heartbeat" ritual that scans the system every few minutes to anticipate user needs without being prompted.

### Token Optimization (Efficiency)
- **Skill Injection**: Backend injects full skill instructions directly into the LLM system prompt, eliminating redundant file reads.
- **Compact Context**: Aggressive history trimming and RAG-based memory recall to keep API costs/latency minimal.

---

## 6. Premium User Experience
- **Interactive Dashboard**: Real-time visualization of consistency scores, behavioral drift, and proactive alerts.
- **Voice-First Journaling**: Built-in speech-to-text for frictionless "Reality Capture."
- **Glassmorphic Design**: Modern, dark-mode aesthetic built with Framer Motion for a premium, alive feel.

---

## 7. Future Roadmap (Hardware Integration)
- **Hardware Telemetry**: Connection to STM32/RPi boards for physical status indicators (e.g., a "Focus LED" that glows when a Skill Block is active).
- **Deeper RAG**: Indexing personal PDF datasheets or academic papers for the Career Agent.

---

## 8. Why We Win (The USPs)
1. **Truly Autonomous**: Mirror doesn't just assist; it recalibrates your day when you fail.
2. **Career-Obsessed**: Directly links daily coding habits to specific role gaps.
3. **Privacy-Native**: Designed for the sovereign engineer who values their data.

---
**Build on Mirror. Powered by Mirror.**
