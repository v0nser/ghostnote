import { createHmac, timingSafeEqual } from "crypto";

import { PLAN_FEATURES, type PlanId } from "@/lib/plans";

type PolarServer = "production" | "sandbox";

export function polarServer(): PolarServer {
  return process.env.POLAR_SERVER === "sandbox" ? "sandbox" : "production";
}

function polarApiBase() {
  return polarServer() === "sandbox" ? "https://sandbox-api.polar.sh" : "https://api.polar.sh";
}

export function getPolarAccessToken() {
  return process.env.POLAR_ACCESS_TOKEN?.trim() || "";
}

export function polarProductId(plan: PlanId) {
  return plan === "team"
    ? process.env.POLAR_PRODUCT_ID_TEAM?.trim() || ""
    : process.env.POLAR_PRODUCT_ID_PRO?.trim() || "";
}

export function isPolarConfigured() {
  return Boolean(getPolarAccessToken() && polarProductId("pro") && polarProductId("team"));
}

async function polarFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const token = getPolarAccessToken();
  if (!token) throw new Error("Polar is not configured.");

  const response = await fetch(`${polarApiBase()}${path}`, {
    ...init,
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
    cache: "no-store",
  });

  const body = await response.text();
  if (!response.ok) {
    throw new Error(body || `Polar request failed (${response.status}).`);
  }
  return (body ? JSON.parse(body) : {}) as T;
}

export type PolarCheckout = {
  id: string;
  status: string;
  url?: string | null;
  customer_email?: string | null;
  metadata?: Record<string, string> | null;
};

export async function createPolarCheckout(input: {
  email: string;
  plan: PlanId;
  successUrl: string;
  returnUrl: string;
}) {
  const productId = polarProductId(input.plan);
  if (!productId) throw new Error(`Missing Polar product id for ${input.plan}.`);

  return polarFetch<PolarCheckout>("/v1/checkouts/", {
    method: "POST",
    body: JSON.stringify({
      products: [productId],
      customer_email: input.email,
      success_url: input.successUrl,
      return_url: input.returnUrl,
      metadata: {
        plan: input.plan,
        email: input.email,
        product: PLAN_FEATURES[input.plan].name,
      },
    }),
  });
}

export async function getPolarCheckout(id: string) {
  return polarFetch<PolarCheckout>(`/v1/checkouts/${id}`);
}

export function isPolarCheckoutPaid(checkout: PolarCheckout) {
  return checkout.status === "succeeded" || checkout.status === "confirmed";
}

function decodeWebhookSecret(secret: string) {
  const raw = secret.startsWith("whsec_") ? secret.slice("whsec_".length) : secret;
  return Buffer.from(raw, "base64");
}

function safeEqual(left: string, right: string) {
  const a = Buffer.from(left);
  const b = Buffer.from(right);
  if (a.length !== b.length) return false;
  return timingSafeEqual(a, b);
}

export function verifyPolarWebhook(rawBody: string, headers: Headers) {
  const secret = process.env.POLAR_WEBHOOK_SECRET?.trim();
  if (!secret) throw new Error("POLAR_WEBHOOK_SECRET is not set.");

  const id = headers.get("webhook-id");
  const timestamp = headers.get("webhook-timestamp");
  const signatureHeader = headers.get("webhook-signature");
  if (!id || !timestamp || !signatureHeader) {
    throw new Error("Missing Polar webhook headers.");
  }

  const expected = createHmac("sha256", decodeWebhookSecret(secret))
    .update(`${id}.${timestamp}.${rawBody}`)
    .digest("base64");

  const valid = signatureHeader.split(" ").some((part) => {
    const signature = part.startsWith("v1,") ? part.slice(3) : part;
    return safeEqual(expected, signature);
  });
  if (!valid) throw new Error("Invalid Polar webhook signature.");
}

export type PolarWebhookEvent = {
  type: string;
  data: {
    id?: string;
    status?: string;
    customer_email?: string | null;
    email?: string | null;
    metadata?: Record<string, string> | null;
    customer?: { email?: string | null } | null;
    checkout_id?: string | null;
    subscription_id?: string | null;
  };
};

export function polarEventEmail(event: PolarWebhookEvent) {
  return (
    event.data.metadata?.email ||
    event.data.customer_email ||
    event.data.customer?.email ||
    event.data.email ||
    ""
  ).trim().toLowerCase();
}

export function polarEventPlan(event: PolarWebhookEvent): PlanId {
  return event.data.metadata?.plan === "team" ? "team" : "pro";
}
