import { NextResponse } from "next/server";

import { getOfferStatus } from "@/lib/store";

export const dynamic = "force-dynamic";

export async function GET() {
  try {
    const status = await getOfferStatus();
    return NextResponse.json(status);
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Could not load offer status." },
      { status: 500 },
    );
  }
}
