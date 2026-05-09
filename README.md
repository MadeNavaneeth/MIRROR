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
  <a href="#-quick-start"><img src="https://img.shields.io/badge/Setup-Fast-green?style=for-the-badge" alt="Setup" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="License" /></a>
</p>

---

## 📺 Demo

https://github.com/MadeNavaneeth/MIRROR/assets/demo.mp4

> **Note:** If the video doesn't play above, you can find the high-quality render here: [**demo.mp4**](demo.mp4)

---

## ✨ What is Mirror?

Mirror is a fast, small, and fully autonomous AI assistant infrastructure designed to run on anything from a **$10 Linux board** to a high-end server. It solves the "Intention-Action Gap" by observing your daily reality and proactively recalibrating your life to match your goals.

- 🏎️ **Ultra-Lightweight:** <5MB RAM footprint (99% smaller than alternatives).
- ⚡ **Instant Boot:** <10ms startup time even on low-frequency edge cores.
- 🔒 **Security First:** 6-digit pairing, workspace-scoped filesystem, and local-only data.
- 🧠 **Sovereign Memory:** Built-in hybrid search (Vector + Keyword) directly in SQLite.

---

## 🧩 The Problem

**Your life is fragmented.** 
- We plan in one app, work in another, and reflect rarely.
- When you miss a task, your plan collapses. Traditional AI doesn't "know" you're failing until you tell it.
- **Mirror fixes this** by being an autonomous observer that heals your schedule in real-time.

---

## 🏛️ Architecture: 4-Agent Coordination

Mirror uses a **Deterministic Blackboard Pipeline (DLBP)** where four specialized agents coordinate via a shared SQLite state:

| Agent | Role | Responsibility |
| :--- | :--- | :--- |
| **Journal** | 📔 The Input | Captures speech/text and classifies Reality events into a sovereign ledger. |
| **Planner** | 📅 The Architect | Dynamically generates your schedule and recalibrates on missed tasks. |
| **Career** | 💼 The Auditor | Maps daily work to job-description skill gaps and flags "Career Risk." |
| **Health** | 🩺 The Constraint | Converts biometric data (sleep/readiness) into hard scheduling limits. |

### 🛠️ Technical Innovations
- **Trait-Based Abstraction:** Every subsystem (Providers, Channels, Tools, Memory) is a trait. Swap Claude for Ollama with one line of TOML.
- **Hybrid Memory:** Custom FTS5 (Keyword) + Vector (Semantic) search built directly into SQLite. No external Vector DB needed.
- **Security-First:** 6-digit pairing codes, workspace-scoped filesystem, and built-in encrypted secrets management.

---

## 📊 Project Presentation (The "PPT")

### The "Intention-Action Gap"
Mirror is designed for a world of fragmented digital data. It solves the drift between your long-term goals and daily behavior.
- **Impact:** Autonomous recovery when tasks are missed.
- **Privacy:** 100% local — zero data leaves your machine.
- **Performance:** 3.4MB binary, <10ms startup. Runs on $10 hardware.

---

## 🚀 Quick Start

### 1. Build & Install
```bash
git clone https://github.com/theonlyhennygod/mirror.git
cd mirror
cargo build --release
cargo install --path .
```

### 2. Onboard
```bash
# Start the interactive 7-step wizard
mirror onboard --interactive
```

### 3. Usage
```bash
# Ask the agent to plan your day
mirror agent -m "I slept 4 hours. Plan a low-intensity coding day focusing on Rust."

# Check system status
mirror status

# Run the long-running autonomous daemon
mirror daemon
```

---

## 🛡️ Security & Privacy

Mirror enforces a strict "No Data Leak" policy:
- **Pairing Required:** 6-digit one-time code for gateway access.
- **Filesystem Scoping:** No access outside your designated workspace.
- **Local-First:** All memory, vector embeddings, and history stay on your machine.
- **Encrypted Secrets:** API keys are stored with local file-level encryption.

---

## 🤖 AI Disclosure

This project was built in collaboration with **Antigravity**, an agentic AI coding assistant. 

AI was used for:
- **Full Identity Rebrand:** Refactoring the entire codebase from ZeroClaw to Mirror.
- **Video Engineering:** Scripting the Remotion terminal simulation.
- **Infrastructure:** Automating security audits and CI/CD workflows.
- **Technical Writing:** Drafting roadmaps and system documentation.

---

## 📜 License
MIT — See [LICENSE](LICENSE) for details.

---
<p align="center">
  <b>Mirror</b> — Reflect your best self. 🦀🪞
</p>
