"use client";

import { type FormEvent, Suspense, useMemo, useState } from "react";
import { useSearchParams } from "next/navigation";
import { Check } from "lucide-react";

import { Footer } from "@/components/footer";
import { Header } from "@/components/header";
import { MagneticButton } from "@/components/magnetic-button";
import { PLAN_FEATURES, type PlanId } from "@/lib/plans";

function CheckoutForm() {
  const params = useSearchParams();
  const plan = (params.get("plan") === "team" ? "team" : "pro") as PlanId;
  const expired = params.get("expired") === "1";
  const canceled = params.get("canceled") === "1";
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const price = expired ? 30 : 15;
  const details = useMemo(() => PLAN_FEATURES[plan], [plan]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      const response = await fetch("/api/checkout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, plan }),
      });
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || "Checkout failed.");
      window.location.href = data.url;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Checkout failed.");
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto grid max-w-5xl gap-10 px-5 py-16 lg:grid-cols-[1.1fr_0.9fr]">
      <div>
        <p className="text-xs uppercase tracking-[0.2em] text-accent-cyan">Checkout</p>
        <h1 className="mt-3 text-4xl font-semibold">{details.name}</h1>
        <p className="mt-3 text-mist">
          {expired
            ? "Early Bird has ended. You are checking out at regular pricing."
            : "Lock the 50% Early Bird rate forever. Paid features unlock as soon as checkout completes."}
        </p>
        {canceled ? <p className="mt-4 text-sm text-rose-300">Checkout was canceled. You can try again below.</p> : null}
        <ul className="mt-8 space-y-2">
          {details.features.map((feature) => (
            <li key={feature} className="flex gap-2 text-sm text-white">
              <Check className="mt-0.5 size-4 text-accent-cyan" /> {feature}
            </li>
          ))}
        </ul>
      </div>
      <form onSubmit={submit} className="rounded-3xl border border-white/10 bg-charcoal p-6">
        <p className="text-sm text-mist">Due today</p>
        <p className="mt-2 text-4xl font-semibold">
          ${price}
          <span className="text-base text-mist">{plan === "team" ? "/user/month" : "/month"}</span>
        </p>
        {!expired ? <p className="mt-1 text-xs text-emerald-300">50% Early Bird discount applied</p> : null}
        <label className="mt-8 block text-sm text-white/80" htmlFor="checkout-email">
          Work email
        </label>
        <input
          id="checkout-email"
          type="email"
          required
          value={email}
          onChange={(event) => setEmail(event.target.value)}
          className="focus-ring mt-2 w-full rounded-full border border-white/15 bg-white/5 px-5 py-3 text-sm text-white"
          placeholder="you@company.com"
        />
        <MagneticButton type="submit" className="mt-6 w-full">
          {busy ? "Redirecting…" : "Continue to payment"}
        </MagneticButton>
        {error ? <p className="mt-3 text-sm text-rose-300">{error}</p> : null}
        <p className="mt-4 text-xs text-white/40">
          If Stripe keys are set, you go to Stripe Checkout. Otherwise GhostNote records the subscription in MongoDB
          and unlocks paid features immediately for this demo.
        </p>
      </form>
    </div>
  );
}

export default function CheckoutPage() {
  return (
    <div>
      <Header />
      <main>
        <Suspense fallback={<p className="px-5 py-16 text-mist">Loading checkout…</p>}>
          <CheckoutForm />
        </Suspense>
      </main>
      <Footer />
    </div>
  );
}
