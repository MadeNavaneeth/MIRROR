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
  <a href="#-documentation--presentation"><img src="https://img.shields.io/badge/PDF-Presentation-blue?style=for-the-badge" alt="Presentation" /></a>
  <a href="#-detailed-setup"><img src="https://img.shields.io/badge/Setup-Fast-green?style=for-the-badge" alt="Setup" /></a>
  <a href="#-license"><img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="License" /></a>
</p>

---

## 📺 Demo

- **[Quick View (25MB demo.mp4)](demo.mp4)** — Optimized for fast loading and GitHub preview.
- **[High-Res Walkthrough (128MB MIRROR.mp4)](https://github.com/MadeNavaneeth/MIRROR/releases)** — Available in the **Releases** section for maximum production quality.

---

## 📄 Documentation & Presentation

For a deep dive into the engineering choices, market analysis, and the vision behind Mirror, please refer to our official presentation:

> [!IMPORTANT]
> **[RV College of Engineering_TeamMirror.pdf](presentation.pdf)**
> *Official submission document for Samsung PRISM / Clash of the Claws.*

---

## ✨ What is Mirror?

Mirror is a fast, small, and fully autonomous AI assistant infrastructure designed for **Sovereign Intelligence**. It solves the "Intention-Action Gap" by being an active observer of your digital life—not just a passive chatbot.

### Key Pillars:
- 🏎️ **Ultra-Lightweight:** Runs on $10 hardware with <5MB RAM (99% smaller than Node.js alternatives).
- ⚡ **Instant Boot:** <10ms startup time. Ready before you finish typing your first command.
- 🔒 **Security First:** 6-digit pairing, workspace-scoped filesystem, and local-only data encryption.
- 🧠 **Sovereign Memory:** Built-in hybrid search (Vector + Keyword) directly in SQLite—no external DB required.

---

## 🏛️ Architecture: The 4-Agent Deterministic Pipeline (DLBP)

Mirror utilizes a **Blackboard Architecture** where specialized agents coordinate through a shared, persistent state.

| Agent | Responsibility | Dynamic Response |
| :--- | :--- | :--- |
| **Journal** 📔 | Captures reality via text/speech. | Indexes your day into a searchable SQLite ledger. |
| **Planner** 📅 | The "Time Architect." | Recalibrates your schedule when you miss a task or drift from your goals. |
| **Career** 💼 | The Skills Auditor. | Compares your output against career goals to flag skill regressions. |
| **Health** 🩺 | The Capability Constraint. | Converts sleep/readiness data into hard limits for the Planner. |

### Why Rust?
We chose Rust to ensure memory safety and zero-cost abstractions, allowing Mirror to run on low-power devices (ARM, RISC-V, STM32) without sacrificing the complexity of autonomous reasoning.

---

## 🛠️ Detailed Setup

### 1. Prerequisites
- **Rust Toolchain:** `rustup` installed.
- **SQLite Development Files:** `libsqlite3-dev`.
- **LLM Access:** An API key (OpenRouter, Anthropic, or OpenAI).

### 2. Installation
```bash
git clone https://github.com/MadeNavaneeth/MIRROR.git
cd MIRROR
cargo build --release
cargo install --path .
```

### 3. Onboarding
```bash
# Enter the interactive 7-step wizard
mirror onboard --interactive
```

---

## 💻 Usage & Help

| Command | Usage |
| :--- | :--- |
| `mirror agent` | Enter a conversation with your Mirror. |
| `mirror status` | View the status of the Blackboard and all active agents. |
| `mirror daemon` | Start the autonomous background runtime. |
| `mirror doctor` | Run full system diagnostics and check provider health. |
| `mirror service install` | Configure Mirror to run as a system daemon (macOS/Linux). |

---

## 🛡️ Security & Privacy
- **6-Digit Pairing:** All new clients must prove physical access by entering a pairing code.
- **Sandboxed Execution:** Tools are strictly scoped to the workspace to prevent path traversal.
- **Encrypted Secrets:** API keys are encrypted at rest using a machine-specific hardware key.

---

## 📜 License
This project is licensed under the **MIT License**.

---
<p align="center">
  <b>Mirror</b> — Reflect your best self. 🦀🪞<br>
  <i>Built with ❤️ by Team Mirror @ RV College of Engineering</i>
</p>
