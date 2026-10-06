import { NextResponse } from "next/server";

import { getPaymentProvider, paymentCopy } from "@/lib/billing";
import { PLAN_FEATURES, type PlanId } from "@/lib/plans";
import { createPolarCheckout } from "@/lib/polar";
import { createSubscription } from "@/lib/store";
import { createStripeCheckout } from "@/lib/stripe";

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
    const provider = getPaymentProvider();

    if (provider === "polar") {
      const checkout = await createPolarCheckout({
        email,
        plan: body.plan,
        successUrl: `${success}&checkout_id={CHECKOUT_ID}`,
        returnUrl: cancel,
      });
      if (!checkout.url) {
        return NextResponse.json({ error: "Polar did not return a checkout URL." }, { status: 502 });
      }
      await createSubscription({
        email,
        plan: body.plan,
        status: "pending",
        provider: "polar",
        polarCheckoutId: checkout.id,
      });
      return NextResponse.json({ url: checkout.url, mode: "polar" });
    }

    if (provider === "stripe") {
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
          provider: "stripe",
          stripeSessionId: session.id,
        });
        return NextResponse.json({ url: session.url, mode: "stripe" });
      }
    }

    if (provider === "none") {
      return NextResponse.json(
        {
          error:
            "Payments are not configured. Add POLAR_ACCESS_TOKEN and Polar product IDs, then redeploy.",
        },
        { status: 503 },
      );
    }

    const { subscription } = await createSubscription({
      email,
      plan: body.plan,
      status: "active",
      provider: "demo",
    });

    return NextResponse.json({
      url: `${success}&code=${encodeURIComponent(subscription.code)}`,
      mode: "demo",
      plan: PLAN_FEATURES[body.plan].name,
      note: paymentCopy("demo"),
    });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Checkout failed." },
      { status: 500 },
    );
  }
}
