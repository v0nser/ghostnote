"use client";

import { type FormEvent, Suspense, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { Check, Lock } from "lucide-react";

import { Footer } from "@/components/footer";
import { Header } from "@/components/header";
import { MagneticButton } from "@/components/magnetic-button";
import { ENTITLEMENT_COPY, type Entitlement } from "@/lib/plans";

type Payload = {
  subscription?: {
    plan: string;
    status: string;
    code: string;
    priceCents: number;
    discountPct: number;
    entitlements: Entitlement[];
  };
  reservation?: { code: string; expiresAt: string };
  entitlements?: { id: Entitlement; label: string; detail: string; unlocked: boolean }[];
};

function AccountBody() {
  const params = useSearchParams();
  const initial = params.get("email") ?? "";
  const [email, setEmail] = useState(initial);
  const [query, setQuery] = useState(initial);
  const [data, setData] = useState<Payload | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!query) return;
    fetch(`/api/account?email=${encodeURIComponent(query)}`)
      .then((res) => res.json())
      .then((payload) => {
        if (payload.error) setError(payload.error);
        else {
          setError("");
          setData(payload);
        }
      })
      .catch(() => setError("Could not load account."));
  }, [query]);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setQuery(email.trim().toLowerCase());
  };

  const unlocked = data?.subscription?.status === "active";

  return (
    <div className="mx-auto max-w-4xl px-5 py-16">
      <h1 className="text-4xl font-semibold">Your GhostNote plan</h1>
      <p className="mt-3 text-mist">Look up the email you used at checkout to see unlocked paid features.</p>

      <form onSubmit={submit} className="mt-8 flex flex-col gap-3 sm:flex-row">
        <input
          type="email"
          required
          value={email}
          onChange={(event) => setEmail(event.target.value)}
          placeholder="you@company.com"
          className="focus-ring min-h-11 flex-1 rounded-full border border-white/15 bg-white/5 px-5 text-sm"
        />
        <MagneticButton type="submit">Load account</MagneticButton>
      </form>
      {error ? <p className="mt-3 text-sm text-rose-300">{error}</p> : null}

      {data?.subscription ? (
        <div className="mt-10 rounded-3xl border border-accent-cyan/30 bg-charcoal p-6">
          <p className="text-xs uppercase tracking-[0.18em] text-accent-cyan">{data.subscription.plan}</p>
          <h2 className="mt-2 text-2xl font-semibold capitalize">{data.subscription.status} subscription</h2>
          <p className="mt-2 text-sm text-mist">
            ${(data.subscription.priceCents / 100).toFixed(0)}/month · {data.subscription.discountPct}% lifetime discount
          </p>
          <p className="mt-2 font-mono text-sm text-white">{data.subscription.code}</p>
        </div>
      ) : query && !error ? (
        <p className="mt-8 text-sm text-mist">No paid plan on this email yet. Reserve or check out from Early Bird.</p>
      ) : null}

      {data?.reservation && !data.subscription ? (
        <p className="mt-6 text-sm text-emerald-300">
          Spot reserved with code {data.reservation.code}. Finish checkout to unlock paid features.
        </p>
      ) : null}

      <div className="mt-10 grid gap-4 md:grid-cols-2">
        {(data?.entitlements?.length
          ? data.entitlements
          : Object.entries(ENTITLEMENT_COPY).map(([id, copy]) => ({
              id: id as Entitlement,
              ...copy,
              unlocked: false,
            }))
        ).map((item) => (
          <article key={item.id} className="rounded-3xl border border-white/10 bg-charcoal p-5">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold">{item.label}</h3>
              {item.unlocked || unlocked ? (
                <Check className="size-4 text-emerald-300" />
              ) : (
                <Lock className="size-4 text-mist" />
              )}
            </div>
            <p className="mt-2 text-sm text-mist">{item.detail}</p>
          </article>
        ))}
      </div>
    </div>
  );
}

export default function AccountPage() {
  return (
    <div>
      <Header />
      <main>
        <Suspense fallback={<p className="px-5 py-16 text-mist">Loading account…</p>}>
          <AccountBody />
        </Suspense>
      </main>
      <Footer />
    </div>
  );
}
