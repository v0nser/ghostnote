import { NextResponse } from "next/server";

import { PLAN_FEATURES, type PlanId } from "@/lib/plans";
import { createSubscription } from "@/lib/store";
import { createStripeCheckout, isStripeConfigured } from "@/lib/stripe";

export const dynamic = "force-dynamic";

function isPlan(value: unknown): value is PlanId {
  return value === "pro" || value === "team";
}

export async function POST(request: Request) {
  try {
    const body = (await request.json()) as { email?: string; plan?: string };
    const email = body.email?.trim().toLowerCase();
    if (!email || !email.includes("@")) {
      return NextResponse.json({ error: "Enter a valid email." }, { status: 400 });
    }
    if (!isPlan(body.plan)) {
      return NextResponse.json({ error: "Choose Pro or Team." }, { status: 400 });
    }

    const origin = new URL(request.url).origin;
    const success = `${origin}/checkout/success?email=${encodeURIComponent(email)}&plan=${body.plan}`;
    const cancel = `${origin}/checkout?plan=${body.plan}&canceled=1`;

    if (isStripeConfigured()) {
      const session = await createStripeCheckout({
        email,
        plan: body.plan,
        successUrl: `${success}&session_id={CHECKOUT_SESSION_ID}`,
        cancelUrl: cancel,
      });
      if (session?.url) {
        await createSubscription({
          email,
          plan: body.plan,
          status: "pending",
          stripeSessionId: session.id,
        });
        return NextResponse.json({ url: session.url, mode: "stripe" });
      }
    }

    const { subscription } = await createSubscription({
      email,
      plan: body.plan,
      status: "active",
    });

    return NextResponse.json({
      url: `${success}&code=${encodeURIComponent(subscription.code)}`,
      mode: "direct",
      plan: PLAN_FEATURES[body.plan].name,
    });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Checkout failed." },
      { status: 500 },
    );
  }
}
