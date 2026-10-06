# Coda

Local-first personal AI agent — and a stealth meeting assistant.

**Coda: Your work, resolved. Your mind, at peace.**

Plan, use tools, remember, and ask before sensitive actions. Meetings still transcribe and answer on your machine, hidden from screen share.

**Repo:** [v0nser/ghostnote](https://github.com/v0nser/ghostnote)  
**Issues:** [github.com/v0nser/ghostnote/issues](https://github.com/v0nser/ghostnote/issues)

## What it does

- **Agent workspace** — goals, plans, tools, approvals, memory, scheduled tasks
- **Stealth meetings** — the window stays out of Zoom, Teams, and Meet screen shares
- **100% local by default** — mic → Whisper → Ollama. Audio does not leave the machine
- **Voice commands** — speak a goal; same local Whisper path
- **Faster replies** — live answers prefer a small local model (`llama3.2:3b`, `qwen2.5:3b`); the agent prefers `llama3.1:8b` for PDFs and tools
- **Sentinel** — high-risk actions always need your approval
- **Windows setup** — in-app model download + [`docs/WINDOWS_SETUP.md`](docs/WINDOWS_SETUP.md)

This repository is three apps:

| Path | What it is |
| --- | --- |
| Repo root | **Coda** desktop app (Tauri + React + Rust) |
| `coda-site/` | Coda marketing site (Astro; comic + simulated demo) |
| `website/` | Older Ghost Note landing (Next.js, Polar/checkout) |

## Data directory and Ghost Note migration

Coda stores data in `~/.coda` (Windows: `%USERPROFILE%\.coda`). Override with `CODA_HOME`.

On first launch it **copies** existing Ghost Note app data from `com.ghostnote.app` into `~/.coda` if that folder exists. The source is **not deleted**. Files Coda already wrote are not overwritten. A marker file `.migrated-from-ghostnote` records that the copy ran.

Speech models live in `~/.coda/models`. Agent state, meeting memory, and logs live under `~/.coda/agent` and `~/.coda/logs`.

## macOS permissions after the rename

The bundle identifier is now `app.coda.desktop`. macOS treats Coda as a **new app**, so previous Ghost Note grants do not apply. Re-allow:

- **Microphone**
- **Screen Recording** (meeting participant audio, and ⌘⇧C / Ctrl+Shift+C screen-ask)
- **Automation** for Mail, Calendar, and Reminders
- **Notifications**

System Settings → Privacy & Security. Approve Automation when macOS prompts after you allow an email, calendar event, or reminder.

## Download vs clone

- **Download** if you want to *use* Coda. Grab the Mac `.dmg` from the landing page. The Windows `.exe` must be built on Windows.
- **Clone** if you want to *build or contribute*. That is `git clone`, `npm install`, and `npm run tauri:dev` — not the installer.

```bash
git clone https://github.com/v0nser/ghostnote.git
cd ghostnote
npm install
```

## Desktop app

### Requirements

- Node.js 20+
- Rust (stable) and [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Ollama](https://ollama.com) running locally, with a **small** model pulled (`ollama pull llama3.2:3b`)
- macOS 13+ for the current `.dmg` (Apple Silicon sidecar is bundled)
- Windows 10/11: WebView2 (auto), then the in-app **Setup** tab or `scripts/setup-whisper.ps1`

### Windows from source

```powershell
npm install
.\scripts\setup-whisper.ps1
ollama pull llama3.2:3b
npm run tauri:dev
```

### Develop

```bash
npm run tauri:dev
```

The UI is Vite on port `1420`. The Rust side captures audio, runs local Whisper via a sidecar, and talks to Ollama at `http://localhost:11434`.

### Meeting action items (local `llama3.1:8b`)

After you hit **Summarize**, Coda extracts structured action items (owner, task, due) from the transcript with local Ollama. It remembers items/topics/people, surfaces overdue or duplicate nudges, and writes an audit line. Email and reminder follow-ups stay drafts until you approve them.

```bash
ollama pull llama3.1:8b
# optional — default is http://127.0.0.1:11434
export OLLAMA_HOST=http://127.0.0.1:11434
ollama serve
```

If Ollama is down, the app reports `Ollama is not reachable at …` and keeps running. If `llama3.1:8b` is missing, extraction fails with `ollama pull llama3.1:8b` — summaries still use the small live-coach model.

**Not included:** cloud fallback, a second agent framework, or a new database crate (memory is `meeting_memory.json` + `meeting_audit.jsonl` under `~/.coda`). Email send and calendar create run **after you approve** in the workspace, using Mail/Outlook and Calendar on this machine.

Agent work needs the larger model:

```bash
ollama pull llama3.1:8b
ollama pull moondream
ollama pull nomic-embed-text
```

Then say things like `summarize the pdf`, `send email to name@mail.com about the recap`, `what's on my screen`, or `schedule a meeting standup tomorrow at 9am`. Approve when asked. macOS will also ask for Mail/Calendar/Reminders automation — allow Coda.

**Screen ask:** ⌘⇧C (Ctrl+Shift+C on Windows) captures the focused window in RAM and streams an answer from `moondream` (or `llava` if you set `CODA_VISION_MODEL`). Frames are not written to disk unless `CODA_SAVE_SCREEN_CAPTURES=1`. This is separate from “take a screenshot”, which still saves a file.

**Memory:** `remember …` stores a fact with a local embedding (`ollama pull nomic-embed-text`). `what do you remember about …` searches memories and open meeting action items. `forget …` lists matches first and deletes rows only after you approve.

**Audit:** the Audit tab lists local events (tasks, approvals, meeting extracts). A human-readable copy lives at `~/.coda/coda_audit.log`. Export downloads that file. Nothing is uploaded.

### Typecheck / UI build

```bash
npm run typecheck
npm run build
```

### Installers

```bash
# macOS .dmg
npm run tauri:build -- --bundles dmg
npm run installers:publish

# Windows .exe — run on Windows
npm run tauri:build
npm run installers:publish
```

`scripts/publish-installers.sh` copies the generated `.dmg` and `.exe` into `website/public/downloads/` and writes `latest.json`.

## Landing site (`coda-site/`)

Hand-drawn Coda page. Copy lives in `coda-site/src/content/site.ts`.

```bash
cd coda-site
npm install
npm run dev
```

Open [http://localhost:4321](http://localhost:4321). Root shortcut: `npm run coda-site`.

## Older Ghost Note landing (`website/`)

```bash
cd website
cp .env.example .env.local
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

From the repo root you can also run `npm run landing`.

### Environment

Copy `website/.env.example`. The live GitHub issues block is wired to this repo:

```
NEXT_PUBLIC_GITHUB_REPO=v0nser/ghostnote
GITHUB_REPO=v0nser/ghostnote
```

| Variable | Required | Purpose |
| --- | --- | --- |
| `NEXT_PUBLIC_GITHUB_REPO` | Yes (defaults to `v0nser/ghostnote`) | Owner/repo shown on the Contribute section |
| `GITHUB_TOKEN` | Optional | Raises GitHub API rate limits for issue polling |
| `MONGODB_URI` | Yes in production | Atlas URI for reservations + subscriptions |
| `MONGODB_DB` | Optional | Database name (default `ghostnote`) |
| `POLAR_ACCESS_TOKEN` | Yes in production | Polar org token — global card checkout |
| `POLAR_PRODUCT_ID_PRO` | Yes in production | Polar product id for Pro |
| `POLAR_PRODUCT_ID_TEAM` | Yes in production | Polar product id for Team |
| `POLAR_WEBHOOK_SECRET` | Yes in production | Unlocks `/account` after payment |
| `STRIPE_*` | Optional | Only if you already have Stripe; Polar is preferred |

Step-by-step Atlas + Polar: [`website/SETUP.md`](website/SETUP.md). Without MongoDB, APIs stay in memory locally. Production will not unlock paid plans unless Polar (or Stripe) is set.

On Vercel, set `NEXT_PUBLIC_GITHUB_REPO=v0nser/ghostnote` (not `ghostnote/ghostnote`) and redeploy. `NEXT_PUBLIC_*` values are baked in at build time.

Checkout routes still exist under `/checkout` but pricing is off the homepage while the local agent ships.

## Contribute

Issues on this page are **live from GitHub**. File one and it shows up in the Contribute section after the next poll (~20s).

1. Open an issue: [new issue](https://github.com/v0nser/ghostnote/issues/new)
2. Fork and branch from `main`
3. Run `npm run typecheck` (desktop) and `npm run build` in `website/` if you touch the landing site
4. Open a pull request

Helpful labels: bugs, docs, design, testing, community.

## Layout

```
├── src/                 # Desktop React UI (meeting + agent workspace)
├── src-tauri/           # Rust: stealth, audio, Whisper, Ollama, agent
├── docs/                # Architecture, security, Windows setup
├── website/             # Next.js landing, APIs, downloads
├── scripts/             # Whisper setup + installer publish
└── .github/workflows/   # Installer release workflow
```

## License

The product site lists Apache-2.0. Add a `LICENSE` file if you want GitHub to detect it.
