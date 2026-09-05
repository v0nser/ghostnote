import { NextResponse } from "next/server";

import { getStripe } from "@/lib/stripe";
import { createSubscription, findSubscription } from "@/lib/store";
import type { PlanId } from "@/lib/plans";

export const dynamic = "force-dynamic";

export async function POST(request: Request) {
  try {
    const body = (await request.json()) as { email?: string; sessionId?: string; plan?: PlanId };
    const email = body.email?.trim().toLowerCase();
    if (!email) {
      return NextResponse.json({ error: "Email is required." }, { status: 400 });
    }

    if (body.sessionId) {
      const stripe = getStripe();
      if (stripe) {
        const session = await stripe.checkout.sessions.retrieve(body.sessionId);
        const plan = (session.metadata?.plan as PlanId | undefined) ?? body.plan ?? "pro";
        const customerEmail = session.customer_email ?? email;
        if (session.payment_status === "paid" || session.status === "complete") {
          const { subscription } = await createSubscription({
            email: customerEmail,
            plan,
            status: "active",
            stripeSessionId: session.id,
          });
          return NextResponse.json({ subscription });
        }
      }
    }

    const existing = await findSubscription(email);
    if (existing) return NextResponse.json({ subscription: existing });

    if (body.plan) {
      const { subscription } = await createSubscription({
        email,
        plan: body.plan,
        status: "active",
      });
      return NextResponse.json({ subscription });
    }

    return NextResponse.json({ error: "No subscription found." }, { status: 404 });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Could not complete checkout." },
      { status: 500 },
    );
  }
}
