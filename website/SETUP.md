# Go live: database + paid plans

Two accounts. About 20 minutes. Polar is the payment path because it works worldwide, does not need a Stripe business review, and pays out to your bank.

No card company lets you skip identity checks entirely. Polar is the lightest version: create an account, add two products, paste five env vars.

## 1. Database — MongoDB Atlas (free)

Used for Early Bird reservations, subscriptions, and the remaining-spots counter.

1. Open [cloud.mongodb.com](https://cloud.mongodb.com) and create a free **M0** cluster.
2. **Database Access** → add a user with a strong password.
3. **Network Access** → Add IP Address → **Allow Access from Anywhere** (`0.0.0.0/0`) so Vercel can connect.
4. **Connect** → **Drivers** → copy the URI. Put the password in, and add `/ghostnote` before the `?`.

```
MONGODB_URI=mongodb+srv://USER:PASSWORD@cluster0.xxxxx.mongodb.net/ghostnote?retryWrites=true&w=majority
MONGODB_DB=ghostnote
```

Collections are created automatically on first reserve/checkout:

- `reservations` (unique `email`)
- `subscriptions` (unique `email`)
- `offerState` (`key: early-bird`)

Locally you can skip Atlas and run Mongo on `127.0.0.1:27017`, or leave `MONGODB_URI` empty (in-memory only; data resets on restart).

## 2. Payments — Polar (recommended)

Polar is a merchant of record. Customers pay Polar with a card (global). Polar handles VAT/GST. Polar then deposits to **your bank account**.

1. Create an account at [polar.sh](https://polar.sh).
2. Create an organization (personal is fine).
3. **Products → Catalogue** → create two recurring monthly products:
   - `GhostNote Early Bird Pro` — **$15 / month**
   - `GhostNote Early Bird Team` — **$15 / month** (or $30 if Early Bird ended)
4. On each product, **⋮ → Copy Product ID**.
5. **Settings → Developers** → create an **Organization Access Token** with `checkouts:write` and `products:read`.
6. After the site is deployed, **Settings → Webhooks** → add:

```
https://YOUR_DOMAIN/api/webhooks/polar
```

Subscribe at least: `checkout.updated`, `order.paid`, `subscription.active`. Copy the webhook secret.

```
POLAR_ACCESS_TOKEN=polar_oat_...
POLAR_PRODUCT_ID_PRO=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
POLAR_PRODUCT_ID_TEAM=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
POLAR_WEBHOOK_SECRET=whsec_...
POLAR_SERVER=production
```

Sandbox testing: create products on Polar sandbox and set `POLAR_SERVER=sandbox`.

Payouts: Polar **Settings → Finance** → add your bank. They will ask for a one-time ID check before the first payout. That is the only “approval.” There is no Stripe-style business wait to start taking cards.

Stripe remains an optional fallback if those keys are set and Polar is not.

## 3. Put the values on Vercel

Vercel project → **Settings → Environment Variables** (Production + Preview):

| Variable | Required |
| --- | --- |
| `MONGODB_URI` | Yes, to persist paid plans |
| `MONGODB_DB` | Optional (`ghostnote`) |
| `POLAR_ACCESS_TOKEN` | Yes, to charge cards |
| `POLAR_PRODUCT_ID_PRO` | Yes |
| `POLAR_PRODUCT_ID_TEAM` | Yes |
| `POLAR_WEBHOOK_SECRET` | Yes, so paid users unlock |
| `POLAR_SERVER` | Optional (`production`) |
| `NEXT_PUBLIC_GITHUB_REPO` | `v0nser/ghostnote` |

Redeploy after saving. `NEXT_PUBLIC_*` values are baked in at build time.

Locally: copy `website/.env.example` → `website/.env.local`, fill the same keys, restart `npm run dev`.

## 4. Check it

```
GET /api/health
```

You want:

```json
{ "ok": true, "mongo": { "ok": true }, "payments": "polar" }
```

Then:

1. Open `/checkout?plan=pro`
2. Pay with a real (or Polar sandbox) card
3. You return to `/checkout/success` and `/account?email=...` shows an **active** plan

Production will **not** unlock plans without Polar or Stripe. Locally, without keys, checkout stays in demo mode (no charge).
