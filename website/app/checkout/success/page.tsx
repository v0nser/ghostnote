"use client";

import { Suspense, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { Check } from "lucide-react";

import { Footer } from "@/components/footer";
import { Header } from "@/components/header";
import { MagneticButton } from "@/components/magnetic-button";
import { ENTITLEMENT_COPY, type Entitlement } from "@/lib/plans";

type AccountPayload = {
  subscription?: {
    plan: string;
    code: string;
    status: string;
    entitlements: Entitlement[];
  };
  error?: string;
};

function SuccessBody() {
  const params = useSearchParams();
  const email = params.get("email") ?? "";
  const plan = params.get("plan") ?? "pro";
  const sessionId = params.get("session_id");
  const [data, setData] = useState<AccountPayload | null>(null);

  useEffect(() => {
    if (!email) return;
    fetch("/api/checkout/complete", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, plan, sessionId }),
    })
      .then((res) => res.json())
      .then((payload) => setData({ subscription: payload.subscription, error: payload.error }))
      .catch(() => setData({ error: "Could not load your subscription." }));
  }, [email, plan, sessionId]);

  const entitlements = data?.subscription?.entitlements ?? [];

  return (
    <div className="mx-auto max-w-3xl px-5 py-16 text-center">
      <p className="text-xs uppercase tracking-[0.2em] text-accent-cyan">You&apos;re in</p>
      <h1 className="mt-3 text-4xl font-semibold">Thank you. Paid features are unlocked.</h1>
      <p className="mt-4 text-mist">
        {email ? (
          <>
            We saved the Early Bird plan for <span className="text-white">{email}</span>.
          </>
        ) : (
          "Your Early Bird subscription is active."
        )}
      </p>
      {data?.subscription?.code ? (
        <p className="mt-4 text-sm text-emerald-300">
          Lifetime discount code: <span className="font-mono text-white">{data.subscription.code}</span>
        </p>
      ) : null}

      <ul className="mt-10 space-y-3 text-left">
        {entitlements.map((id) => (
          <li key={id} className="flex gap-3 rounded-2xl border border-white/10 bg-charcoal p-4">
            <Check className="mt-0.5 size-4 text-accent-cyan" />
            <div>
              <p className="text-sm font-medium">{ENTITLEMENT_COPY[id].label}</p>
              <p className="mt-1 text-sm text-mist">{ENTITLEMENT_COPY[id].detail}</p>
            </div>
          </li>
        ))}
      </ul>

      <div className="mt-10 flex flex-wrap justify-center gap-3">
        <MagneticButton href={email ? `/account?email=${encodeURIComponent(email)}` : "/account"}>
          Open my account
        </MagneticButton>
        <MagneticButton href="/#cta" variant="secondary">
          Download GhostNote
        </MagneticButton>
      </div>
    </div>
  );
}

export default function SuccessPage() {
  return (
    <div>
      <Header />
      <main>
        <Suspense fallback={<p className="px-5 py-16 text-mist">Confirming your plan…</p>}>
          <SuccessBody />
        </Suspense>
      </main>
      <Footer />
    </div>
  );
}
