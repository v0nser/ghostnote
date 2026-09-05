import { NextResponse } from "next/server";

import { ENTITLEMENT_COPY } from "@/lib/plans";
import { findReservation, findSubscription } from "@/lib/store";

export const dynamic = "force-dynamic";

export async function GET(request: Request) {
  const email = new URL(request.url).searchParams.get("email")?.trim().toLowerCase();
  if (!email) {
    return NextResponse.json({ error: "Email is required." }, { status: 400 });
  }

  const [subscription, reservation] = await Promise.all([findSubscription(email), findReservation(email)]);
  return NextResponse.json({
    subscription,
    reservation,
    entitlements: (subscription?.entitlements ?? []).map((id) => ({
      id,
      ...ENTITLEMENT_COPY[id],
      unlocked: subscription?.status === "active",
    })),
  });
}
