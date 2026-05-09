<p align="center">
  <img src="mirror.png" alt="Mirror" width="250" />
</p>

<h1 align="center">Mirror 🪞</h1>

<p align="center">
  <strong>The Sovereign Personal Intelligence System.</strong><br>
  Zero overhead. Zero compromise. 100% Rust. 100% Private.
</p>

<p align="center">
  <a href="#-demo"><img src="https://img.shields.io/badge/Demo-Watch%20Now-e05c2a?style=for-the-badge" alt="Demo" /></a>
  <a href="#-detailed-setup"><img src="https://img.shields.io/badge/Setup-Fast-green?style=for-the-badge" alt="Setup" /></a>
  <a href="#-license"><img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="License" /></a>
</p>

---

## 📺 Demo

https://github.com/MadeNavaneeth/MIRROR/assets/demo.mp4

> **Note:** If the video doesn't play above, you can find the high-quality render here: [**demo.mp4**](demo.mp4)

---

## ✨ What is Mirror?

Mirror is an ultra-fast, small, and fully autonomous AI assistant infrastructure designed for the next generation of sovereign computing. Whether running on a **$10 Linux board** or a high-end workstation, Mirror observes your daily reality and proactively recalibrates your schedule to match your intentions.

- 🏎️ **Ultra-Lightweight:** <5MB RAM footprint (99% smaller than alternatives).
- ⚡ **Instant Boot:** <10ms startup time even on low-frequency edge cores.
- 🔒 **Security First:** 6-digit pairing, workspace-scoped filesystem, and local-only data.
- 🧠 **Sovereign Memory:** Built-in hybrid search (Vector + Keyword) directly in SQLite.

---

## 🧩 The Problem: The "Intention-Action Gap"

Most productivity tools are passive. You plan, but life happens.
- **Fragmentation:** Data is locked in disparate apps.
- **Drift:** Your behavior silently drifts from your career and health goals.
- **Fragility:** If you miss one task, your entire plan for the week collapses.

**Mirror fixes this** by being an autonomous observer that uses a **Deterministic Blackboard Pipeline (DLBP)** to heal your schedule in real-time.

---

## 🏛️ Architecture: The 4-Agent Coordination

Mirror coordinates four specialized agents via a shared SQLite "Blackboard" state:

1. **Journal Agent (📔 The Input):** Captures ambient data (speech/text) and classifies events into a sovereign ledger.
2. **Planner Agent (📅 The Architect):** Proactively generates and recalibrates your schedule based on reality updates.
3. **Career Agent (💼 The Auditor):** Compares daily "Proof-of-Work" against professional goals to flag skill gaps.
4. **Health Agent (🩺 The Constraint):** Converts biometrics (sleep/readiness) into hard "Capability Constraints" for the Planner.

---

## 🛠️ Detailed Setup

### Prerequisites
- **Rust:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **SQLite:** `libsqlite3-dev` (Linux) or standard install (macOS/Windows).
- **API Key:** An OpenRouter or OpenAI API key for LLM processing.

### Installation
```bash
git clone https://github.com/MadeNavaneeth/MIRROR.git
cd MIRROR
cargo build --release
cargo install --path .
```

### Onboarding
Start the interactive 7-step wizard to configure your identity, providers, and security policy:
```bash
mirror onboard --interactive
```

---

## 💻 Usage Help

| Command | Description |
| :--- | :--- |
| `mirror agent` | Enter interactive chat mode with your Mirror. |
| `mirror status` | Check the health of all agents and the background daemon. |
| `mirror daemon` | Start the autonomous runtime (Heartbeat & Background Tasks). |
| `mirror doctor` | Diagnose connection issues or configuration errors. |
| `mirror service install` | Install Mirror as a background service (macOS/Linux). |

### Example Proactive Interaction
> **User:** "I slept 4 hours. Plan a low-intensity coding day."
>
> **Mirror:** "Recognized Health Constraint. Recalibrating Planner... Deep work shifted to 2 PM. High-intensity meetings deferred to tomorrow."

---

## 🛡️ Security & Privacy
- **Pairing Code:** Every new device must pair via a 6-digit code shown in the CLI.
- **Workspace Scoping:** Agents are strictly forbidden from accessing paths outside the repository.
- **Encrypted Secrets:** Your API keys are encrypted at rest using your local machine ID.

---

## 🤖 AI Disclosure
This project was developed in collaboration with **Antigravity**, an agentic AI assistant. AI was utilized for:
- **Project Rebrand:** Systematic context-aware refactoring of 100+ files.
- **Video Engineering:** Scripting the Remotion-based terminal simulation.
- **Technical Writing:** Authoring the architecture and security roadmaps.

---

## 📜 License

This project is licensed under the **MIT License**.

```text
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions...
```
See the [LICENSE](LICENSE) file for the full text.

---
<p align="center">
  <b>Mirror</b> — Reflect your best self. 🦀🪞
</p>
