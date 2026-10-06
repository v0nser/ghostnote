import { NextResponse } from "next/server";

import { getPaymentProvider, paymentCopy } from "@/lib/billing";
import { pingDb } from "@/lib/mongodb";

export const dynamic = "force-dynamic";

export async function GET() {
  const mongo = await pingDb();
  const payments = getPaymentProvider();
  return NextResponse.json({
    ok: (mongo.ok || payments === "demo") && payments !== "none",
    mongo,
    payments,
    copy: paymentCopy(payments),
  });
}
