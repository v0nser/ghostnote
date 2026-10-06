import { NextResponse } from "next/server";

import {
  polarEventEmail,
  polarEventPlan,
  verifyPolarWebhook,
  type PolarWebhookEvent,
} from "@/lib/polar";
import { createSubscription } from "@/lib/store";

export const dynamic = "force-dynamic";

const PAID_EVENTS = new Set([
  "checkout.updated",
  "order.paid",
  "subscription.created",
  "subscription.active",
]);

export async function POST(request: Request) {
  const raw = await request.text();
  try {
    verifyPolarWebhook(raw, request.headers);
  } catch {
    return NextResponse.json({ error: "Invalid Polar signature." }, { status: 400 });
  }

  const event = JSON.parse(raw) as PolarWebhookEvent;
  if (!PAID_EVENTS.has(event.type)) {
    return NextResponse.json({ received: true });
  }

  if (event.type === "checkout.updated" && event.data.status && !["succeeded", "confirmed"].includes(event.data.status)) {
    return NextResponse.json({ received: true });
  }

  const email = polarEventEmail(event);
  if (!email) return NextResponse.json({ received: true });

  await createSubscription({
    email,
    plan: polarEventPlan(event),
    status: "active",
    provider: "polar",
    polarCheckoutId: event.data.checkout_id ?? event.data.id ?? undefined,
    polarSubscriptionId: event.data.subscription_id ?? undefined,
  });

  return NextResponse.json({ received: true });
}
