# Implementation progress

Living status for the Ghost Note agent platform. Updated after each phase.

## Phase

Phase 1–5 foundation + product UX (homepage, Windows setup, faster replies, voice).

## Status

Complete for the foundation. Later phases are stubbed with explicit boundaries.

## Architectural decision

The spec’s FastAPI/Python gateway is **not** being added. Ghost Note already has a
Tauri/Rust IPC boundary. The agent orchestrator, tools, Sentinel, memory, and
approvals live in `src-tauri/src/agent/`. Logical HTTP-style routes are Tauri
commands (`agent_create_task`, `agent_approve`, …).

Meeting capture, stealth, Whisper, and the live interview coach are unchanged.

## Changes

- Architecture and security docs under `docs/`
- Agent orchestrator with an explicit state machine and local JSON persistence
- Tool registry, Ghost Sentinel, approvals, permissions, memory, planner
- Agent workspace + command center + Setup tab in the existing Tauri UI
- Voice commands reuse the mic + Whisper pipeline
- Faster local model routing for live replies (`llama3.2:3b` / `qwen2.5:3b` first)
- In-app Whisper download + `scripts/setup-whisper.ps1`
- Homepage: pricing and Early Bird removed; capabilities of the shipping platform shown
- Meeting summaries can spawn follow-up agent tasks

## Tests

- `cargo test --lib` — 49 passed (state machine helpers, Sentinel, permissions, planner, SSRF, path jail, memory, model picker)
- `npm run typecheck` — desktop UI
- `npx tsc --noEmit` in `website/` — landing page

## Known issues

- Browser click/type need a future Playwright sidecar (Phase 6)
- Computer-use is an API stub defaulting to DENY (Phase 10)
- Cloud model providers are interfaces only
- Scheduled tasks run only while the app is open
- Sandbox is a process jail, not a VM

## Next step

Phase 6: isolated Playwright browser context, then sandbox hardening and
connector implementations.
