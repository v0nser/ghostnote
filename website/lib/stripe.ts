import Stripe from "stripe";

import { PLAN_FEATURES, priceForPlan, type PlanId } from "@/lib/plans";

export function getStripe() {
  const key = process.env.STRIPE_SECRET_KEY;
  if (!key) return null;
  return new Stripe(key, { apiVersion: "2024-06-20" });
}

export function isStripeConfigured() {
  return Boolean(process.env.STRIPE_SECRET_KEY && process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY);
}

export async function createStripeCheckout(input: {
  email: string;
  plan: PlanId;
  successUrl: string;
  cancelUrl: string;
}) {
  const stripe = getStripe();
  if (!stripe) return null;

  const pricing = priceForPlan(input.plan, false);
  const session = await stripe.checkout.sessions.create({
    mode: "subscription",
    customer_email: input.email,
    success_url: input.successUrl,
    cancel_url: input.cancelUrl,
    allow_promotion_codes: false,
    metadata: {
      plan: input.plan,
      earlyBird: "true",
    },
    line_items: [
      {
        quantity: 1,
        price_data: {
          currency: "usd",
          recurring: { interval: "month" },
          unit_amount: pricing.monthlyCents,
          product_data: {
            name: PLAN_FEATURES[input.plan].name,
            description: "GhostNote Early Bird — 50% off locked forever",
          },
        },
      },
    ],
  });

  return session;
}
