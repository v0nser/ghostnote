# GhostNote

Local-first stealth AI meeting assistant. Transcribe, summarize, and get a spoken-style answer on your machine — and stay hidden while you share your screen.

**Repo:** [v0nser/ghostnote](https://github.com/v0nser/ghostnote)  
**Issues:** [github.com/v0nser/ghostnote/issues](https://github.com/v0nser/ghostnote/issues)

## What it does

- **Stealth mode** — the window stays out of Zoom, Teams, and Meet screen shares
- **100% local AI** — mic → Whisper → Ollama. Audio does not leave the machine
- **Real-time answers** — streams a first-person reply when the other person stops talking
- **Short context** — only the last ~45 seconds go to the model
- **Your models** — tuned for `llama3.1:latest`; Mistral and Qwen work the same day you pull them

This repository is two apps:

| Path | What it is |
| --- | --- |
| Repo root | GhostNote **desktop app** (Tauri + React + Rust) |
| `website/` | Marketing / Early Bird / checkout site (Next.js) |

## Download vs clone

- **Download** if you want to *use* GhostNote. Grab the Mac `.dmg` from the landing page (`/downloads/GhostNote.dmg`). The Windows `.exe` must be built on Windows.
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
- [Ollama](https://ollama.com) running locally, with a model pulled (e.g. `ollama pull llama3.1`)
- macOS 13+ for the current `.dmg` (Apple Silicon sidecar is bundled). Windows needs a Windows whisper sidecar + NSIS build.

### Develop

```bash
npm run tauri:dev
```

The UI is Vite on port `1420`. The Rust side captures audio, runs local Whisper via a sidecar, and talks to Ollama at `http://localhost:11434`.

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

`scripts/publish-installers.sh` copies `GhostNote.dmg` and `GhostNote-Setup.exe` into `website/public/downloads/` and writes `latest.json`.

## Landing site (`website/`)

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
| `MONGODB_URI` | Optional | Persist Early Bird reservations and subscriptions |
| `MONGODB_DB` | Optional | Database name (default `ghostnote`) |
| `STRIPE_SECRET_KEY` | Optional | Real Stripe Checkout instead of the local path |
| `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY` | Optional | Stripe.js on the client |
| `STRIPE_WEBHOOK_SECRET` | Optional | `/api/webhooks/stripe` |

Without MongoDB the APIs still run in memory for local demos. Without Stripe, checkout writes an active subscription locally and unlocks `/account`.

On Vercel, set `NEXT_PUBLIC_GITHUB_REPO=v0nser/ghostnote` (not `ghostnote/ghostnote`) and redeploy. `NEXT_PUBLIC_*` values are baked in at build time.

### Paid plans

- Early Bird: 50% off (`$30` → `$15` / month), 500 spots
- Checkout: `/checkout?plan=pro` or `/checkout?plan=team`
- Account: `/account`

## Contribute

Issues on this page are **live from GitHub**. File one and it shows up in the Contribute section after the next poll (~20s).

1. Open an issue: [new issue](https://github.com/v0nser/ghostnote/issues/new)
2. Fork and branch from `main`
3. Run `npm run typecheck` (desktop) and `npm run build` in `website/` if you touch the landing site
4. Open a pull request

Helpful labels: bugs, docs, design, testing, community.

## Layout

```
├── src/                 # Desktop React UI
├── src-tauri/           # Rust: stealth, audio, Whisper sidecar, Ollama
├── website/             # Next.js landing, APIs, downloads
├── scripts/             # Publish .dmg / .exe into website/public/downloads
└── .github/workflows/   # Installer release workflow
```

## License

The product site lists Apache-2.0. Add a `LICENSE` file if you want GitHub to detect it.
