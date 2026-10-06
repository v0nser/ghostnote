# Ghost Note Architecture

Local-first personal AI agent platform. The meeting assistant is one capability,
not the whole product.

This document describes the **current** desktop architecture, the **proposed**
agent architecture, and how they fit together. It does not copy or claim access
to any proprietary system.

## Current architecture

Ghost Note is a **Tauri 2** desktop app. There is no Python service and no
FastAPI process. Meeting intelligence already runs as Rust modules behind
Tauri IPC, with a React UI.

```
┌─────────────────────────────────────────────┐
│  React UI (Vite)                            │
│  Dashboard · Stealth pill · Zustand stores  │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC + events
                   ▼
┌─────────────────────────────────────────────┐
│  Rust (src-tauri)                           │
│  stealth · audio · transcribe · session     │
│  ollama/coach                               │
└──────────┬──────────────────┬───────────────┘
           │                  │
           ▼                  ▼
   whisper-cli sidecar    Ollama (localhost:11434)
   ggml-base.en.bin       llama / qwen / mistral
```

| Area | What exists |
| --- | --- |
| Desktop shell | Tauri 2, custom chrome, transparent window, tray-ready |
| Frontend | React 19, Vite, Zustand, Tailwind |
| Stealth | macOS ScreenCaptureKit exclusion, Windows capture exclusion, notification silence, pill mode |
| Audio | cpal microphone; macOS ScreenCaptureKit + Windows WASAPI loopback |
| Transcription | whisper.cpp sidecar, serial queue, VAD segmentation |
| LLM | Ollama HTTP client, streamed interview answers, meeting summary |
| Context | Last ~45 seconds of transcript to the live model |
| Events | `ghostnote://transcript-segment`, talking-points, coach-status, VAD, levels |
| Cancellation | Generation id + oneshot abort for in-flight Ollama calls |
| Persistence | None for meetings. Whisper model lives in the app data dir |
| Auth | None in the desktop app |
| Website | Next.js marketing site, Polar/Stripe checkout, GitHub issues |
| Tests | Rust unit tests in coach, transcribe, parse |
| Packaging | macOS `.dmg`, Windows NSIS, `scripts/setup-whisper.sh` (Unix only) |

### Reusable components

- Tauri command/event bus (the Agent Gateway)
- Ollama client + streaming + model warm-up
- Whisper sidecar + utterance pipeline (voice commands reuse this)
- Audio capture with mic-only option
- Zustand + IPC frontend pattern
- Stealth window (unchanged)
- Local-only logging policy (never log user content)

### Conflicts with a naive port of the spec

The written spec sketches FastAPI, WebSockets, and HTTP routes such as
`POST /agents`. Standing up a Python service would be a second product and
would break the current IPC security boundary.

**Decision (no product wait):** implement the agent platform **inside Rust**,
and expose the same logical API as Tauri commands. The meeting pipeline stays
exactly where it is.

## Proposed architecture

```
                    ┌──────────────────────┐
                    │      Ghost Note      │
                    │      Tauri UI        │
                    └──────────┬───────────┘
                               │ invoke / events
                               ▼
                    ┌──────────────────────┐
                    │   Agent Gateway      │
                    │   (Tauri commands)   │
                    └──────────┬───────────┘
                               ▼
                    ┌──────────────────────┐
                    │   Agent Orchestrator │
                    │   explicit states    │
                    └───────┬───────┬──────┘
                            │       │
                ┌───────────┘       └────────────┐
                ▼                                ▼
        ┌──────────────┐                 ┌───────────────┐
        │ Model Router │                 │ Memory System │
        └──────┬───────┘                 └───────────────┘
               │
        Ollama first · optional remote later

                      Agent
          ┌───────────┼──────────────┐
          ▼           ▼              ▼
       Browser     Filesystem      Sandbox
        Tools        Tools          Tools
                      │
                      ▼
             ┌──────────────────┐
             │ Ghost Sentinel   │
             │ Policy Engine    │
             └────────┬─────────┘
                      │
             approval required?
                 /          \
               YES           NO
                │             │
          User approval    execute
```

Meeting Assistant remains a first-class profile. After a meeting, detected
action items can become agent tasks with one click.

## Components

### Reuse

- `stealth`, `audio`, `transcribe`, `session`, `ollama::Coach`
- Frontend dashboard and stealth pill
- Website download + contribute flows

### Modify

- `ollama::Client` → shared **Model Router** (fast local models first)
- `session` → optional voice-command capture (mic only)
- Dashboard shell → workspace with Meeting / Agent / Setup views
- Landing page → ship the platform, hide pricing for now
- Windows setup → in-app wizard + PowerShell script + model download

### Add

| Module | Role |
| --- | --- |
| `agent::orchestrator` | State machine, pause/resume/cancel/retry |
| `agent::planner` | Goal → structured plan, replan on failure |
| `agent::tools` | Registry + builtin tools |
| `agent::sentinel` | Deterministic risk / approval policy |
| `agent::approvals` | Human checkpoints |
| `agent::permissions` | Global / agent / task / tool grants |
| `agent::memory` | Working, episodic, semantic, preferences |
| `agent::store` | Local JSON persistence (survives restart) |
| `agent::audit` | Append-only event log, no secrets |
| `agent::sandbox` | Isolated temp workspace, allowlisted commands |
| `agent::computer` | Desktop control **default DENY** |
| `agent::scheduler` | One-shot / interval / daily / weekly |
| `agent::connectors` | Interface + local notes reference |
| `agent::voice` | Speak a goal into the orchestrator |
| `agent::setup` | First-run checklist, Whisper download |

## Data flow

1. User types or speaks a goal.
2. Gateway creates a task (`IDLE` → `UNDERSTANDING` → `PLANNING`).
3. Context engine pulls only relevant memory, plan, and tool schemas.
4. Planner writes steps with tools, dependencies, and risk.
5. Orchestrator executes safe steps through the tool registry.
6. Ghost Sentinel evaluates every side-effect. High/critical → approval UI.
7. Tools emit events. UI shows action summaries, never hidden chain-of-thought.
8. Artifacts, memory updates, and audit rows are written locally.
9. Task reaches `COMPLETED`, `FAILED`, `CANCELLED`, or `PAUSED`.

Meeting path is unchanged:

```
mic / system audio → VAD → Whisper → coach.consider → streamed answer
```

After stop + summarize, action items can spawn tasks.

## Persistence

All agent data stays under the Tauri app data directory:

```
{app_data}/com.ghostnote.app/
  models/ggml-base.en.bin
  agent/
    config.json
    tasks/{task_id}.json
    memories.json
    permissions.json
    approvals.json
    agents.json
    schedules.json
    audit.jsonl
    workspace/          # default filesystem root
    artifacts/
    sandboxes/{id}/
```

No remote sync unless the user later configures a connector.

## Security boundaries

| Boundary | Rule |
| --- | --- |
| UI → Rust | Tauri IPC only. Frontend cannot execute tools. |
| LLM → tools | Model proposes calls. Sentinel decides. |
| Tools → host | Workspace jail. No `~/.ssh`, no credentials store. |
| Tools → network | http/https only. Localhost / link-local / metadata IPs blocked. |
| Sandbox → OS | Allowlisted interpreters, CPU/time/output limits. |
| Computer use | Default DENY. Per-task opt-in later. |
| Cloud models | Off by default. Banner when enabled. |
| Logs | No passwords, tokens, cookies, payment data, or raw meeting audio. |

Ghost Sentinel is deterministic code. The model cannot grant itself permissions.

## Website

`website/` is marketing and checkout only. It does not host the agent runtime.
Pricing is removed from the homepage while the platform ships as a local app.
