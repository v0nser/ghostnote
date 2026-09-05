import { NextResponse } from "next/server";

import { getGitHubBundle } from "@/lib/github";

export const dynamic = "force-dynamic";

export async function GET() {
  const data = await getGitHubBundle();
  return NextResponse.json(data, {
    headers: {
      "Cache-Control": "no-store, max-age=0",
    },
  });
}
