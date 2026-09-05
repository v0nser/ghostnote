import { randomUUID } from "crypto";

import {
  DEFAULT_CLAIMED,
  EARLY_BIRD_MONTHLY_CENTS,
  PLAN_FEATURES,
  TOTAL_EARLY_BIRD_SPOTS,
  makeDiscountCode,
  type Entitlement,
  type PlanId,
} from "@/lib/plans";
import { getDb, isMongoConfigured } from "@/lib/mongodb";

export type Reservation = {
  id: string;
  email: string;
  code: string;
  createdAt: string;
  expiresAt: string;
};

export type Subscription = {
  id: string;
  email: string;
  plan: PlanId;
  status: "active" | "pending" | "canceled";
  priceCents: number;
  discountPct: number;
  code: string;
  entitlements: Entitlement[];
  stripeSessionId?: string;
  createdAt: string;
};

type OfferState = {
  claimed: number;
  last24h: number;
};

type MemoryStore = {
  reservations: Reservation[];
  subscriptions: Subscription[];
  offer: OfferState;
};

const globalForStore = globalThis as typeof globalThis & { __ghostnoteStore?: MemoryStore };

const memory =
  globalForStore.__ghostnoteStore ??
  ({
    reservations: [],
    subscriptions: [],
    offer: { claimed: DEFAULT_CLAIMED, last24h: 41 },
  } satisfies MemoryStore);

globalForStore.__ghostnoteStore = memory;

function remainingFrom(claimed: number) {
  return Math.max(0, TOTAL_EARLY_BIRD_SPOTS - claimed);
}

export async function getOfferStatus() {
  const db = await getDb();
  if (db) {
    const doc = await db.collection<OfferState & { _id: string }>("offerState").findOne({ _id: "early-bird" });
    const claimed = doc?.claimed ?? DEFAULT_CLAIMED;
    const last24h = doc?.last24h ?? 41;
    if (!doc) {
      await db.collection("offerState").insertOne({
        _id: "early-bird",
        claimed: DEFAULT_CLAIMED,
        last24h: 41,
      });
    }
    return {
      claimed,
      remaining: remainingFrom(claimed),
      last24h,
      persistence: "mongodb" as const,
    };
  }

  return {
    claimed: memory.offer.claimed,
    remaining: remainingFrom(memory.offer.claimed),
    last24h: memory.offer.last24h,
    persistence: isMongoConfigured() ? ("unavailable" as const) : ("memory" as const),
  };
}

export async function reserveSpot(email: string) {
  const normalized = email.trim().toLowerCase();
  if (!normalized.includes("@")) {
    throw new Error("Enter a valid email.");
  }

  const now = new Date();
  const reservation: Reservation = {
    id: randomUUID(),
    email: normalized,
    code: makeDiscountCode(),
    createdAt: now.toISOString(),
    expiresAt: new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000).toISOString(),
  };

  const db = await getDb();
  if (db) {
    const existing = await db.collection("reservations").findOne({ email: normalized });
    if (existing) {
      return {
        reservation: {
          id: String(existing.id ?? existing._id),
          email: existing.email as string,
          code: existing.code as string,
          createdAt: existing.createdAt as string,
          expiresAt: existing.expiresAt as string,
        },
        existing: true,
      };
    }
    await db.collection("reservations").insertOne(reservation);
    await db.collection("offerState").updateOne(
      { _id: "early-bird" },
      { $inc: { claimed: 1, last24h: 1 }, $setOnInsert: { claimed: DEFAULT_CLAIMED, last24h: 41 } },
      { upsert: true },
    );
    return { reservation, existing: false };
  }

  const already = memory.reservations.find((item) => item.email === normalized);
  if (already) return { reservation: already, existing: true };
  memory.reservations.push(reservation);
  memory.offer.claimed += 1;
  memory.offer.last24h += 1;
  return { reservation, existing: false };
}

export async function createSubscription(input: {
  email: string;
  plan: PlanId;
  stripeSessionId?: string;
  status?: Subscription["status"];
}) {
  const email = input.email.trim().toLowerCase();
  const plan = input.plan;
  const existing = await findSubscription(email);

  if (existing?.status === "active") {
    return { subscription: existing, created: false };
  }

  const subscription: Subscription = {
    id: randomUUID(),
    email,
    plan,
    status: input.status ?? "active",
    priceCents: EARLY_BIRD_MONTHLY_CENTS,
    discountPct: 50,
    code: existing?.code ?? makeDiscountCode(),
    entitlements: PLAN_FEATURES[plan].entitlements,
    stripeSessionId: input.stripeSessionId,
    createdAt: new Date().toISOString(),
  };

  const db = await getDb();
  if (db) {
    await db.collection("subscriptions").updateOne(
      { email },
      { $set: subscription },
      { upsert: true },
    );
    if (!existing) {
      await db.collection("offerState").updateOne(
        { _id: "early-bird" },
        { $inc: { claimed: 1, last24h: 1 } },
        { upsert: true },
      );
    }
    return { subscription, created: !existing };
  }

  const index = memory.subscriptions.findIndex((item) => item.email === email);
  if (index >= 0) memory.subscriptions[index] = subscription;
  else {
    memory.subscriptions.push(subscription);
    memory.offer.claimed += 1;
    memory.offer.last24h += 1;
  }
  return { subscription, created: index < 0 };
}

export async function findSubscription(email: string) {
  const normalized = email.trim().toLowerCase();
  const db = await getDb();
  if (db) {
    const doc = await db.collection("subscriptions").findOne({ email: normalized });
    return doc ? (doc as unknown as Subscription) : null;
  }
  return memory.subscriptions.find((item) => item.email === normalized) ?? null;
}

export async function findReservation(email: string) {
  const normalized = email.trim().toLowerCase();
  const db = await getDb();
  if (db) {
    const doc = await db.collection("reservations").findOne({ email: normalized });
    return doc ? (doc as unknown as Reservation) : null;
  }
  return memory.reservations.find((item) => item.email === normalized) ?? null;
}
