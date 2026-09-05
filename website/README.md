# GhostNote landing

Marketing site for GhostNote. Separate from the Tauri desktop app.

```bash
cd website
cp .env.example .env.local
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

## Paid plans

Checkout lives at `/checkout?plan=pro` or `/checkout?plan=team`. Reservations, subscriptions, and Early Bird spot counts persist in MongoDB when `MONGODB_URI` is set. Without MongoDB the same APIs keep working in memory for local demos.

If Stripe keys are present, checkout creates a Stripe subscription session. Otherwise the success page writes an active subscription to MongoDB and unlocks paid entitlements on `/account`.

## Desktop installers

`Download for Mac` serves `public/downloads/GhostNote.dmg`. Build it from the repo root:

```bash
npm run tauri:build -- --bundles dmg
npm run installers:publish
```

The Windows `.exe` must be built on Windows (`npm run tauri:build`), then the same publish script copies `GhostNote-Setup.exe` next to the dmg.

## Live GitHub issues

Set `NEXT_PUBLIC_GITHUB_REPO=owner/repo` in `.env.local`. The Contribute section polls GitHub every 20 seconds and shows real open issues only — no sample tickets. Testers can file issues from the page; after GitHub accepts them they appear in the list.
