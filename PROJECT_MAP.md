# Mirror Project Map (Where is what?)

This file is a **detailed map** of the repository: which folders/files exist, what they do, and where to change things.

## Quick orientation (the 80/20)

- **CLI entrypoint**: `src/main.rs`
- **Library crate exports (module index)**: `src/lib.rs`
- **Agent orchestration loop**: `src/agent/`
- **Gateway server (HTTP API + serves dashboard build)**: `src/gateway/mod.rs`
- **Model providers (OpenAI/Anthropic/OpenRouter/Ollama + compatible)**: `src/providers/`
- **Channels (Telegram/Discord/Slack/WhatsApp/etc.)**: `src/channels/`
- **Tools (shell/file/memory/browser/etc.)**: `src/tools/`
- **Security policy + pairing + secret store + sandboxing**: `src/security/`
- **Memory backends (sqlite/lucid/markdown/none + embeddings)**: `src/memory/`
- **Background daemon (runs gateway + channels + heartbeat + scheduler)**: `src/daemon/mod.rs`
- **Scheduling (cron DB + scheduler loop)**: `src/cron/`
- **React dashboard UI**: `dashboard/`

## Repo layout (top-level)

- `src/`: Rust implementation (the product).
- `dashboard/`: React + Vite dashboard UI (built to `dashboard/dist` and served by the gateway).
- `docs/`: deeper architecture and operational docs (security, sandboxing, deployment, hardware).
- `skills/`: **repo-local skill packs** (content for Mirror “skills” ecosystem).
- `scripts/`: helper scripts (build/import utilities).
- `examples/`: example usage / samples.
- `tests/` + `test_helpers/`: integration tests and test helpers.
- `firmware/`: firmware-related assets for hardware/peripheral work.
- `dev/`: dev/CI scripts (see `AGENTS.md` for recommended validation).

## Top-level files (what is each one for?)

### Core project metadata

- `Cargo.toml`: Rust crate manifest (dependencies, features like `hardware`, release profile tuned for size).
- `Cargo.lock`: exact dependency lock for reproducible builds.
- `rust-toolchain.toml`: pins the Rust toolchain for consistent builds.
- `.cargo/config.toml`: cargo build settings (linker/flags/etc.).
- `deny.toml`: cargo-deny policy (licenses/advisories/bans/sources).

### Primary docs (product / engineering)

- `README.md`: main overview, quick start, architecture summary, configuration examples.
- `AGENTS.md`: engineering protocol and architecture guardrails (trait+factory structure, security-first areas, validation matrix).
- `docs/design/design-system.md`: UI/design system direction (visual theme tokens and principles).
- `SECURITY.md`: vulnerability reporting + security posture overview.
- `CONTRIBUTING.md`: contribution process and expectations.
- `CHANGELOG.md`: user-facing change history (if maintained).
- `RUN_TESTS.md`: how to run the test suites (telegram-focused in this repo snapshot).
- `TESTING_TELEGRAM.md`: deeper Telegram testing and troubleshooting.

### Identity / “OpenClaw-style” workspace context

These are consumed by the system prompt builder in `src/channels/mod.rs` (OpenClaw format), and/or by AIEOS loading (`src/identity.rs`).

- `SOUL.md`: personality/values guidance.
- `HEARTBEAT.md`: periodic task list for heartbeat engine.
- `PROACTIVE_LOG.md`: output log for proactive scans (written by proactive/heartbeat tooling).
- (Optional) `BOOTSTRAP.md`: only injected if present (first-run ritual).
- `MEMORY.md`: curated “core memory” file (also used by snapshot/hydration flows).

### Deployment / packaging

- `Dockerfile`: container build for running gateway/daemon in a hardened container.
- `docker-compose.yml`: local composition (if used for dev/demo).

### Mirror (project variant on top of Mirror)

These describe “Mirror” (a multi-agent application built on Mirror).

- `docs/mirror/README_Mirror.md`: Mirror concept and 4-agent architecture overview.
- `docs/mirror/MIRROR_DETAILED_PROPOSAL.md`: detailed product/architecture writeup.
- `docs/mirror/MIRROR_SLIDE_DECK.md` + `docs/mirror/HACKATHON_DECK_SOURCE.md`: deck content.

### OpenCode reference notes

- `docs/opencode/DESIGN.md`: reference design notes pulled from OpenCode-inspired UI tokens.

## How the program starts (entrypoints)

### `mirror` CLI

- **Routing + commands**: `src/main.rs`
  - Commands like `agent`, `gateway`, `daemon`, `status`, `cron`, `channel`, `skills`, `plugins`, `auth` are parsed here.
  - `onboard` runs **before config load**; most other commands load config first.

### Library exports

- **Public module tree**: `src/lib.rs`
  - Re-exports `Config` (`src/config/`) and subcommand enums used by the CLI.

## Core runtime modules (what does what)

### Agent loop (`src/agent/`)

Goal: **turn an input message into an LLM response**, optionally calling tools in a loop.

Key files:
- `src/agent/loop_.rs`: message processing + tool-call loop implementation (the “brain loop”).
- `src/agent/agent.rs`: `Agent` / builder style orchestration.
- `src/agent/dispatcher.rs`: routing between provider/tool/memory/observer.
- `src/agent/memory_loader.rs`: loads workspace/memory context used by the agent prompt path.
- `src/agent/prompt.rs`: prompt assembly utilities for agent mode.
- `src/agent/reasoning.rs`: reasoning configuration / scaffolding used by the loop.
- `src/agent/mod.rs`: module exports + re-exports.

### Configuration (`src/config/`)

Goal: define **all config schema + defaults + env overrides**.

Key files:
- `src/config/schema.rs`: the actual typed config structs (what you can set in `config.toml`).
- `src/config/mod.rs`: re-exports (so other modules can import `crate::config::Config`, etc.).

### Providers (LLM backends) (`src/providers/`)

Goal: offer a single trait (`Provider`) that many providers implement, plus resilience wrappers.

Key files:
- `src/providers/traits.rs`: `Provider` trait + message/response structs.
- `src/providers/mod.rs`: **factory** (`create_provider`, `create_resilient_provider`, routing) + API error sanitization.
- Provider implementations:
  - `src/providers/openrouter.rs`
  - `src/providers/anthropic.rs`
  - `src/providers/openai.rs`
  - `src/providers/gemini.rs`
  - `src/providers/ollama.rs` (local)
  - `src/providers/compatible.rs` (OpenAI-compatible endpoints)
  - `src/providers/reliable.rs` (retry/fallback wrapper)
  - `src/providers/router.rs` (model routing)

### Tools (capabilities) (`src/tools/`)

Goal: define “things the agent can do” (shell/file/memory/http/browser/etc.) behind a `Tool` trait.

Key files:
- `src/tools/traits.rs`: `Tool`, `ToolSpec`, `ToolResult`.
- `src/tools/mod.rs`: tool registry builders
  - `default_tools(...)` is minimal (`shell`, `file_read`, `file_write`)
  - `all_tools(...)` / `all_tools_with_runtime(...)` adds memory tools, scheduling, git ops, browser/http (when enabled), composio (when enabled), delegate (when configured)

Tool implementations (each file is one “tool”):
- `src/tools/shell.rs`: run commands via runtime adapter + security allowlists.
- `src/tools/file_read.rs`: read files with workspace scoping.
- `src/tools/file_write.rs`: write files with workspace scoping.
- `src/tools/git_operations.rs`: git helper operations (scoped to workspace).
- `src/tools/memory_store.rs`: store memory via memory backend.
- `src/tools/memory_recall.rs`: recall/search memory.
- `src/tools/memory_forget.rs`: delete memory entries.
- `src/tools/schedule.rs`: create/list/cancel scheduled tasks (backs onto `src/cron/` storage).
- `src/tools/browser_open.rs`: “open URL” tool (allowlist-only).
- `src/tools/browser.rs`: full browser automation abstraction (multiple backends).
- `src/tools/http_request.rs`: HTTP fetch tool (allowlist + limits).
- `src/tools/screenshot.rs`: screenshot capture (policy-gated).
- `src/tools/image_info.rs`: inspect image metadata (policy-gated).
- `src/tools/composio.rs`: Composio integration tool (optional via config).
- `src/tools/delegate.rs`: delegate subtasks to configured sub-agents.
- Hardware-facing tools:
  - `src/tools/hardware_board_info.rs`
  - `src/tools/hardware_memory_map.rs`
  - `src/tools/hardware_memory_read.rs`
- Project-specific:
  - `src/tools/gog.rs`: tool wired to auth handler for GOG-related actions.

### Channels (message transports) (`src/channels/`)

Goal: receive messages from external systems and feed them into the agent loop; send responses back.

Key files:
- `src/channels/traits.rs`: `Channel` trait and `ChannelMessage` shape.
- `src/channels/mod.rs`: starts channels, supervises listeners, parallelizes message processing, builds system prompt.

Channel implementations:
- `src/channels/telegram.rs`
- `src/channels/discord.rs`
- `src/channels/slack.rs`
- `src/channels/whatsapp.rs`
- `src/channels/matrix.rs`
- `src/channels/imessage.rs`
- `src/channels/email_channel.rs`
- `src/channels/irc.rs`
- `src/channels/lark.rs`
- `src/channels/dingtalk.rs`
- `src/channels/cli.rs` (always available)

Notes:
- `src/channels/mod.rs` is the “channel runtime”: it builds the system prompt, does memory context injection, runs the tool-call loop, sends replies back to the correct channel, and supervises listeners with backoff.

### Gateway (HTTP server) (`src/gateway/`)

Goal: a local-first HTTP server for:
- pairing + bearer tokens
- webhook chat
- WhatsApp webhook
- dashboard APIs
- serving the built dashboard (`dashboard/dist`)

Key file:
- `src/gateway/mod.rs`: axum server with body limits, timeouts, rate limiting, idempotency, pairing, secret masking for `/api/config`.

Important endpoints (see code for exact behavior):
- `GET /health`
- `POST /pair`
- `POST /webhook`
- `GET /whatsapp` + `POST /whatsapp`
- `GET/POST /api/config`
- `GET /api/cost`
- `POST /api/chat`
- `GET /api/auth/status`
- `GET/POST /api/memories`

Dashboard serving:
- The gateway fallback serves `dashboard/dist` (static build). For local dev UI, you typically run Vite separately via `dashboard/package.json` scripts.

### Security (`src/security/`)

Goal: keep the agent safe-by-default:
- pairing + auth tokens
- file/workspace scoping
- sandbox detection/creation
- secret encryption at rest
- audit logging

Key files:
- `src/security/policy.rs`: `SecurityPolicy` (autonomy levels, allowlists, forbidden paths, etc.)
- `src/security/pairing.rs`: pairing code + token verification utilities
- `src/security/secrets.rs`: `SecretStore` (encrypt/decrypt)
- `src/security/audit.rs`: audit events / logger
- `src/security/detect.rs`: sandbox selection
- `src/security/traits.rs`: `Sandbox` trait + no-op sandbox
- Optional sandboxes/adapters:
  - `src/security/docker.rs`
  - `src/security/landlock.rs` (linux + feature)
  - `src/security/bubblewrap.rs` (feature)
  - `src/security/firejail.rs` (linux)

### Memory (`src/memory/`)

Goal: persistence + retrieval. Supports multiple backends and a hybrid search approach.

Key files:
- `src/memory/traits.rs`: `Memory` trait, `MemoryEntry`, `MemoryCategory`.
- `src/memory/mod.rs`: **factory** (`create_memory`, migration helper, response cache).
- Backends:
  - `src/memory/sqlite.rs` (main durable backend)
  - `src/memory/lucid.rs` (hybrid: external “lucid” + sqlite fallback)
  - `src/memory/markdown.rs`
  - `src/memory/none.rs`
- Search/embeddings/hygiene:
  - `src/memory/embeddings.rs`
  - `src/memory/vector.rs`
  - `src/memory/hygiene.rs`
  - `src/memory/snapshot.rs` (snapshot + hydrate)
- Supporting:
  - `src/memory/backend.rs`: backend classification + UX labels.
  - `src/memory/chunker.rs`: chunking logic for markdown / ingestion.
  - `src/memory/response_cache.rs`: optional response cache.

### Runtime adapters (`src/runtime/`)

Goal: abstract “how commands are executed” (native vs docker sandbox).

Key files:
- `src/runtime/traits.rs`: `RuntimeAdapter` trait
- `src/runtime/mod.rs`: factory (`create_runtime`)
- `src/runtime/native.rs`
- `src/runtime/docker.rs`
- `src/runtime/wasm.rs`: WASM runtime placeholder/experimental surface.

### Daemon (long-running) (`src/daemon/`)

Goal: keep components alive with supervision and backoff.

Key file:
- `src/daemon/mod.rs`: spawns supervisors for `gateway`, `channels`, `heartbeat`, `scheduler` and periodically writes `daemon_state.json`.

### Cron / scheduler (`src/cron/`)

Goal: persistent scheduled tasks (cron + one-shot) stored in SQLite + a scheduler loop.

Key files:
- `src/cron/mod.rs`: CLI handlers + cron DB implementation (`cron/jobs.db` under workspace).
- `src/cron/scheduler.rs`: polling/execution loop.

### Heartbeat (`src/heartbeat/`)

Goal: periodic “maintenance prompts” driven by `HEARTBEAT.md`.

Key files:
- `src/heartbeat/engine.rs`: parses/collects tasks and orchestrates runs.
- `src/heartbeat/proactive.rs`: proactive scanning engine (logs to `PROACTIVE_LOG.md`).
- `src/heartbeat/mod.rs`: exports.

### Plugins (`src/plugins/`)

Goal: extension system for add-ons (plugin metadata, registry, enable/disable/install).

Key files:
- `src/plugins/manager.rs`: `PluginManager`
- `src/plugins/registry.rs`: registry lookup
- `src/plugins/loader.rs`: plugin loading
- `src/plugins/traits.rs`: plugin trait contracts
- `src/plugins/mod.rs`: exports.

### Auth / OAuth (`src/auth/`)

Goal: provider sign-in flows + credential storage (encrypted-at-rest via `SecretStore`).

Key files:
- `src/auth/handler.rs`: `AuthHandler` (login/logout/list)
- `src/auth/storage.rs`: credential storage implementation
- `src/auth/oauth.rs`: OAuth helper flows
- `src/auth/providers.rs`: provider-specific auth wiring
- `src/auth/mod.rs`: exports.

### Observability + health (`src/observability/`, `src/health/`)

Goal: make runtime measurable and debuggable.

Health:
- `src/health/mod.rs`: component status snapshots (used by `/health` and daemon state writer).

Observer implementations:
- `src/observability/traits.rs`: `Observer` trait.
- `src/observability/noop.rs`: no-op observer.
- `src/observability/log.rs`: simple structured logging observer.
- `src/observability/verbose.rs`: more verbose logging.
- `src/observability/multi.rs`: fan-out to multiple observers.
- `src/observability/otel.rs`: OpenTelemetry exporter wiring.
- `src/observability/mod.rs`: exports + factory.

## Other `src/` modules you’ll run into

### Identity (`src/identity.rs`)

Goal: support identity configuration formats:
- “OpenClaw style” workspace markdown injection (AGENTS/SOUL/TOOLS/IDENTITY/USER/HEARTBEAT/MEMORY).
- AIEOS JSON identity (file or inline) converted into a system prompt.

### Cost tracking (`src/cost/`)

- `src/cost/tracker.rs`: records per-request token usage and computes summaries (used by CLI `status` and gateway `/api/cost`).
- `src/cost/types.rs`: token usage structs.
- `src/cost/mod.rs`: exports and glue.

### Integrations registry (`src/integrations/`)

- `src/integrations/mod.rs`: CLI handler for `mirror integrations ...`.
- `src/integrations/registry.rs`: integration catalog data / lookup.

### Hardware discovery (`src/hardware/`)

- `src/hardware/mod.rs`: CLI wiring for hardware commands.
- `src/hardware/discover.rs`: USB enumeration / discovery.
- `src/hardware/introspect.rs`: inspect a device path and extract details.
- `src/hardware/registry.rs`: known board registry/mapping.

### Peripherals (`src/peripherals/`)

This is “hardware the agent can control” (serial boards, rpi gpio, firmware flows).

- `src/peripherals/mod.rs`: CLI wiring and factory.
- `src/peripherals/traits.rs`: `Peripheral` trait contracts.
- `src/peripherals/serial.rs`: serial transport utilities.
- `src/peripherals/rpi.rs`: Raspberry Pi GPIO implementation (linux feature gated).
- `src/peripherals/arduino_flash.rs`: Arduino flash/setup flow.
- `src/peripherals/arduino_upload.rs`: upload sketches/payloads.
- `src/peripherals/nucleo_flash.rs`: Nucleo flash flow (probe-rs feature gated).
- `src/peripherals/capabilities_tool.rs`: tool exposure for peripherals.
- `src/peripherals/uno_q_bridge.rs` + `src/peripherals/uno_q_setup.rs`: Arduino Uno Q bridge helpers.

### Tunnel providers (`src/tunnel/`)

- `src/tunnel/mod.rs`: tunnel trait + factory.
- `src/tunnel/none.rs`: no tunnel.
- `src/tunnel/cloudflare.rs`: Cloudflare tunnel adapter.
- `src/tunnel/tailscale.rs`: Tailscale tunnel adapter.
- `src/tunnel/ngrok.rs`: ngrok adapter.
- `src/tunnel/custom.rs`: custom tunnel command adapter.

### Onboarding + doctor + service

- `src/onboard/mod.rs` + `src/onboard/wizard.rs`: interactive setup, models refresh/list helpers.
- `src/doctor/mod.rs`: diagnostics command implementation.
- `src/service/mod.rs`: install/start/stop/status/uninstall OS service wrapper.

### Migration (`src/migration.rs`)

- Handles migration commands (e.g. importing from other runtimes like OpenClaw).

### RAG (`src/rag/`)

- `src/rag/mod.rs`: retrieval-augmented generation surfaces (PDF ingestion is feature-gated in `Cargo.toml`).

### Skillforge (`src/skillforge/`)

This is a “skill workflow” subsystem (scouting/evaluating/integrating skills).

- `src/skillforge/mod.rs`
- `src/skillforge/scout.rs`
- `src/skillforge/evaluate.rs`
- `src/skillforge/integrate.rs`

### Utilities (`src/util.rs`)

- Common helpers used across modules (e.g., safe string truncation utilities used in channel/gateway logs).

## Dashboard UI (`dashboard/`)

This is a separate React app (Vite) used as a dashboard front-end.

- Entry: `dashboard/index.html`, `dashboard/src/main.tsx`, `dashboard/src/App.tsx`
- Build output: `dashboard/dist/` (served by gateway fallback in `src/gateway/mod.rs`)
- Local dev:
  - `cd dashboard && npm install`
  - `npm run dev`

Key UI files in this repo snapshot:
- `dashboard/src/App.tsx`: main shell / routes.
- `dashboard/src/Settings.tsx`: settings panel UI.
- `dashboard/src/mirror.ts`: Mirror-specific UI/data wiring.
- `dashboard/src/config-utils.ts`: helpers for talking to `/api/config` and sanitizing config edits.
- `dashboard/src/types.ts`: shared types used across UI.

Note: the current `dashboard/README.md` is still the default Vite template readme (not Mirror-specific yet).

## “Skills” content (`skills/`)

These are **skill packs** included in this repo (documentation + scripts + metadata). They are not the same as the runtime `src/skills/mod.rs` loader, but they’re meant to be consumed by it.

Typical shape:
- `skills/<name>/SKILL.md`: skill instructions
- `skills/<name>/_meta.json` and `.clawdhub/origin.json`: skill metadata
- `skills/<name>/scripts/*`: helper scripts
- `skills/<name>/references/*`: reference docs for the skill (optional)

Examples in this repo:
- `skills/agentmail/` (email automation; scripts in `skills/agentmail/scripts/`)
- `skills/auto-memory/` (memory automation; shell scripts under `skills/auto-memory/scripts/`)
- `skills/auto-respawn/` (wallet/transfer automation; TypeScript package inside)
- `skills/obsidian-direct/` (Obsidian helpers; python scripts)
- `skills/gog/` (GOG tool skill)

## Firmware (`firmware/`)

Hardware support includes firmware projects:
- `firmware/mirror-esp32/`: ESP32 firmware (Rust).
- `firmware/mirror-nucleo/`: Nucleo firmware (Rust).

## Scripts (`scripts/`)

Repo helper scripts (build/import/test utilities), e.g.:
- `scripts/build_ui.sh`
- `scripts/import_to_mirror.py`
- `scripts/build_uiux_kb.py`

## Docs (`docs/`)

Good starting points:
- **Security & sandboxing**: `docs/sandboxing.md`, `docs/agnostic-security.md`, `docs/frictionless-security.md`
- **Network / deployment**: `docs/network-deployment.md`
- **Hardware**: `docs/hardware-peripherals-design.md`, `docs/nucleo-setup.md`, `docs/arduino-uno-q-setup.md`
- **CI / contribution process**: `docs/ci-map.md`, `docs/pr-workflow.md`, `docs/reviewer-playbook.md`

## Tests

- Rust unit tests live alongside modules.
- Integration tests live under `tests/`.
- Quick commands: see `RUN_TESTS.md` (and `AGENTS.md` validation matrix).

## Adding / extending things (common paths)

- **Add a new provider**: implement `Provider` in `src/providers/` and register in `src/providers/mod.rs`.
- **Add a new channel**: implement `Channel` in `src/channels/` and wire it in `src/channels/mod.rs` + config schema.
- **Add a new tool**: implement `Tool` in `src/tools/` and add it to `all_tools_with_runtime(...)` in `src/tools/mod.rs`.
- **Change security rules**: `src/security/policy.rs` and relevant tool/channel enforcement points.
- **Add config keys**: `src/config/schema.rs` (treat as public API; document defaults/migrations).

## Common “where do I change X?” shortcuts

- **Add a new CLI command**:
  - Add clap wiring in `src/main.rs`
  - If the command is shared/exported: add enum wiring in `src/lib.rs`
  - Put the implementation in a new module (common patterns: `src/doctor/`, `src/service/`, `src/onboard/`)

- **Change the system prompt (identity injection / bootstrap files)**:
  - Main builder: `src/channels/mod.rs` (`build_system_prompt`, file injection list)
  - AIEOS conversion: `src/identity.rs`
  - Workspace files involved (created/used at runtime): `AGENTS.md`, `SOUL.md`, `TOOLS.md`, `IDENTITY.md`, `USER.md`, `HEARTBEAT.md`, `MEMORY.md`
  - Note: `TOOLS.md`, `IDENTITY.md`, `USER.md`, `MEMORY.md` may be created by onboarding; they might not exist in this git repo.

- **Change gateway endpoints or security constraints**:
  - HTTP handlers + middleware: `src/gateway/mod.rs`
  - Pairing/token rules: `src/security/pairing.rs`
  - “refuse public bind” logic: `src/security/pairing.rs` (public bind detection) and `src/gateway/mod.rs` (enforcement)

- **Add/modify dashboard API responses**:
  - Server-side: `src/gateway/mod.rs` (e.g. `/api/config`, `/api/cost`, `/api/chat`, `/api/memories`)
  - Client-side: `dashboard/src/config-utils.ts`, `dashboard/src/types.ts`, and the relevant UI component

- **Add a new provider auth flow**:
  - CLI command wiring: `src/main.rs` (`AuthCommands`)
  - Handler: `src/auth/handler.rs`
  - Storage/encryption: `src/auth/storage.rs` + `src/security/secrets.rs`
  - Provider-specific logic: `src/auth/providers.rs` and `src/auth/oauth.rs`

- **Add a new “tool” capability**:
  - Implement: new file under `src/tools/` implementing `Tool` (`src/tools/traits.rs`)
  - Register: `src/tools/mod.rs` (add to `all_tools_with_runtime`)
  - Enforce security: `src/security/policy.rs` + the tool’s own checks

- **Change what the daemon runs / supervision**:
  - Supervisor/backoff + which components start: `src/daemon/mod.rs`
  - Component health snapshots: `src/health/mod.rs`

- **Change scheduling semantics**:
  - Storage + CLI: `src/cron/mod.rs`
  - Scheduler loop: `src/cron/scheduler.rs`
  - Tool interface: `src/tools/schedule.rs`

- **Change memory backend behavior**:
  - Backend selection/factory: `src/memory/mod.rs`, `src/memory/backend.rs`
  - SQLite details: `src/memory/sqlite.rs`
  - Embeddings + vector merge: `src/memory/embeddings.rs`, `src/memory/vector.rs`
  - Hygiene + snapshots: `src/memory/hygiene.rs`, `src/memory/snapshot.rs`

- **Change channel runtime behavior (parallelism, timeouts, allowlists)**:
  - Channel runtime loop + timeouts: `src/channels/mod.rs`
  - Specific channel logic: `src/channels/<channel>.rs`

- **Add a new tunnel provider**:
  - Implement: `src/tunnel/<provider>.rs`
  - Register: `src/tunnel/mod.rs`
  - Enforce security assumptions: gateway refuses public bind without tunnel unless explicitly allowed (`src/gateway/mod.rs`)

---

If you want it even more exhaustive, I can generate a second file (for example `PROJECT_MAP_FILES.md`) that is literally a **full tree listing** with 1–2 lines per file. This `PROJECT_MAP.md` aims to be detailed but still readable.

