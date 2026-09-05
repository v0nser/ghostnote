import { NextResponse } from "next/server";

import { reserveSpot } from "@/lib/store";

export const dynamic = "force-dynamic";

export async function POST(request: Request) {
  try {
    const body = (await request.json()) as { email?: string };
    if (!body.email) {
      return NextResponse.json({ error: "Email is required." }, { status: 400 });
    }
    const result = await reserveSpot(body.email);
    return NextResponse.json({
      ok: true,
      existing: result.existing,
      code: result.reservation.code,
      expiresAt: result.reservation.expiresAt,
    });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Could not reserve a spot." },
      { status: 400 },
    );
  }
}
