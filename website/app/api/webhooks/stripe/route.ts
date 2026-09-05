import { NextResponse } from "next/server";
import type Stripe from "stripe";

import { createSubscription } from "@/lib/store";
import { getStripe } from "@/lib/stripe";
import type { PlanId } from "@/lib/plans";

export const dynamic = "force-dynamic";

export async function POST(request: Request) {
  const stripe = getStripe();
  const secret = process.env.STRIPE_WEBHOOK_SECRET;
  if (!stripe || !secret) {
    return NextResponse.json({ error: "Stripe webhook is not configured." }, { status: 501 });
  }

  const signature = request.headers.get("stripe-signature");
  if (!signature) {
    return NextResponse.json({ error: "Missing signature." }, { status: 400 });
  }

  const payload = await request.text();
  let event: Stripe.Event;
  try {
    event = stripe.webhooks.constructEvent(payload, signature, secret);
  } catch {
    return NextResponse.json({ error: "Invalid signature." }, { status: 400 });
  }

  if (event.type === "checkout.session.completed") {
    const session = event.data.object as Stripe.Checkout.Session;
    const email = session.customer_email;
    const plan = (session.metadata?.plan as PlanId | undefined) ?? "pro";
    if (email) {
      await createSubscription({
        email,
        plan,
        status: "active",
        stripeSessionId: session.id,
      });
    }
  }

  return NextResponse.json({ received: true });
}
