"use client";

import { type FormEvent, useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
  Check,
  ChevronDown,
  Globe,
  Lock,
  Shield,
  Sparkles,
  X,
  Zap,
} from "lucide-react";

import { CountdownTimer } from "@/components/countdown-timer";
import { MagneticButton } from "@/components/magnetic-button";
import { TiltCard } from "@/components/tilt-card";
import { useCountdown } from "@/hooks/use-countdown";
import { PLAN_FEATURES } from "@/lib/plans";
import { cn } from "@/lib/utils";

const FAQ = [
  {
    q: "Is this really 50% off forever?",
    a: "Yes, your discount is locked in for life. The Early Bird rate follows you to higher tiers.",
  },
  {
    q: "Can I upgrade later?",
    a: "Yes, and your discount applies to higher tiers too. Pro to Team keeps the same 50% lock.",
  },
  {
    q: "What if I don't like it?",
    a: "30-day money-back guarantee, no questions asked. Cancel from the account page.",
  },
];

const PARTICLES = Array.from({ length: 24 }, (_, index) => ({
  id: index,
  left: `${(index * 37) % 100}%`,
  top: `${(index * 53) % 100}%`,
  delay: `${(index % 8) * 0.4}s`,
  size: 2 + (index % 3),
}));

export function EarlyBird() {
  const { expired } = useCountdown();
  const [claimed, setClaimed] = useState(347);
  const [remaining, setRemaining] = useState(153);
  const [email, setEmail] = useState("");
  const [code, setCode] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [openFaq, setOpenFaq] = useState<number | null>(0);

  useEffect(() => {
    fetch("/api/early-bird/status")
      .then((res) => res.json())
      .then((data) => {
        if (typeof data.claimed === "number") setClaimed(data.claimed);
        if (typeof data.remaining === "number") setRemaining(data.remaining);
        if (typeof data.last24h === "number") setClaimed((value) => Math.max(value, data.last24h + 300));
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    const id = window.setInterval(() => {
      setClaimed((value) => value + (Math.random() > 0.6 ? 1 : 0));
    }, 4000);
    return () => window.clearInterval(id);
  }, []);

  const filled = Math.min(100, Math.round(((500 - remaining) / 500) * 100));

  const reserve = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      const response = await fetch("/api/early-bird/reserve", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || "Could not reserve.");
      setCode(data.code);
      setRemaining((value) => Math.max(0, value - (data.existing ? 0 : 1)));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not reserve.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section id="early-bird" className="relative overflow-hidden px-5 py-20 md:py-[80px]">
      <div className="early-bird-aurora pointer-events-none absolute inset-0" />
      {PARTICLES.map((particle) => (
        <span
          key={particle.id}
          className="pointer-events-none absolute rounded-full bg-accent-cyan/40 animate-float-slow"
          style={{
            left: particle.left,
            top: particle.top,
            width: particle.size,
            height: particle.size,
            animationDelay: particle.delay,
          }}
        />
      ))}

      <div className="relative mx-auto max-w-[1200px] rounded-[32px] border border-accent-cyan/30 bg-black/40 p-6 shadow-[0_0_0_1px_rgba(34,211,238,0.12)] md:p-12">
        <div className="pointer-events-none absolute inset-0 rounded-[32px] early-bird-border" />

        <div className="relative text-center">
          <motion.h2
            initial={{ opacity: 0, y: -24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="early-bird-headline text-3xl font-bold md:text-5xl"
          >
            {expired ? "Early Bird offer ended" : "🚀 Early Bird Special: 50% OFF Lifetime Access"}
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.5 }}
            className="mx-auto mt-4 max-w-2xl text-lg text-white/80 md:text-2xl"
          >
            {expired
              ? "You missed the launch window. Regular pricing is live — Pro is $30/month."
              : "Join the first 500 users and lock in exclusive pricing forever"}
          </motion.p>
        </div>

        <div className="relative mt-10 flex justify-center">
          {expired ? (
            <p className="rounded-full border border-rose-400/40 bg-rose-500/10 px-5 py-2 text-sm text-rose-200">
              Offer expired
            </p>
          ) : (
            <CountdownTimer />
          )}
        </div>

        <div className="relative mt-12 grid items-stretch gap-5 lg:grid-cols-3">
          <article className="relative opacity-50">
            <TiltCard className="h-full animate-subtle-shake">
              <p className="text-sm uppercase tracking-[0.16em] text-white/50">Regular Price</p>
              <p className="mt-4 text-4xl font-semibold text-white/70 line-through">$30/month</p>
              <p className="mt-2 text-sm text-mist">After Early Bird</p>
              <span className="pointer-events-none absolute inset-0 flex items-center justify-center">
                <X className="size-24 text-rose-500/70" strokeWidth={1.25} />
              </span>
            </TiltCard>
          </article>

          <article className="relative z-10 lg:scale-110">
            <TiltCard className="h-full border-accent-cyan/50 bg-black/60 shadow-[0_0_40px_rgba(34,211,238,0.16)]">
              <p className="absolute -top-3 left-6 animate-pulse rounded-full bg-accent-cyan px-3 py-0.5 text-[10px] font-semibold text-ink">
                BEST VALUE
              </p>
              <p className="text-sm uppercase tracking-[0.16em] text-accent-cyan">Early Bird — PRO</p>
              <p className="mt-3 text-sm text-white/45 line-through">$30/month</p>
              <p className="mt-1 text-5xl font-bold text-accent-cyan">$15/month</p>
              <p className="mt-3 inline-flex rounded-full bg-emerald-400/15 px-3 py-1 text-xs font-semibold text-emerald-300">
                Save $180/year
              </p>
              <ul className="mt-6 space-y-2 text-sm">
                {PLAN_FEATURES.pro.features.map((feature) => (
                  <li key={feature} className="flex gap-2 text-white">
                    <Check className="mt-0.5 size-4 text-accent-cyan" /> {feature}
                  </li>
                ))}
              </ul>
              <MagneticButton
                href={expired ? "/checkout?plan=pro&expired=1" : "/checkout?plan=pro"}
                className="shimmer-btn mt-8 w-full"
              >
                {expired ? "Regular Pricing" : "Claim 50% OFF Now"}
              </MagneticButton>
            </TiltCard>
          </article>

          <article>
            <TiltCard className="h-full border-accent-purple/40">
              <p className="text-sm uppercase tracking-[0.16em] text-accent-purple">Early Bird — TEAM</p>
              <p className="mt-3 text-sm text-white/45 line-through">$30/user/month</p>
              <p className="mt-1 text-4xl font-bold text-white">$15/user/month</p>
              <p className="mt-3 inline-flex rounded-full bg-accent-purple/20 px-3 py-1 text-xs font-semibold text-accent-purple">
                Save 50% Forever
              </p>
              <ul className="mt-6 space-y-2 text-sm">
                {PLAN_FEATURES.team.features.map((feature) => (
                  <li key={feature} className="flex gap-2 text-white">
                    <Check className="mt-0.5 size-4 text-accent-purple" /> {feature}
                  </li>
                ))}
              </ul>
              <MagneticButton href="/checkout?plan=team" variant="secondary" className="mt-8 w-full">
                Get Team Discount
              </MagneticButton>
            </TiltCard>
          </article>
        </div>

        <p className="relative mt-14 text-center text-sm text-white/80">
          🔥 {claimed} people claimed this offer in the last 24 hours
        </p>

        <div className="relative mx-auto mt-6 max-w-xl">
          <div className="mb-2 flex justify-between text-xs text-white/70">
            <span>Spots remaining</span>
            <span>Only {remaining} spots left at this price</span>
          </div>
          <div className="h-2 overflow-hidden rounded-full bg-white/10">
            <motion.div
              className="h-full rounded-full bg-gradient-to-r from-accent-cyan via-accent-purple to-accent-blue"
              initial={{ width: 0 }}
              whileInView={{ width: `${filled || 69}%` }}
              viewport={{ once: true }}
              transition={{ duration: 1.1 }}
            />
          </div>
        </div>

        <form onSubmit={reserve} className="relative mx-auto mt-10 flex max-w-xl flex-col gap-3 sm:flex-row">
          <label className="sr-only" htmlFor="early-bird-email">
            Email
          </label>
          <input
            id="early-bird-email"
            type="email"
            required
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            placeholder="Enter email to lock in your discount"
            className="focus-ring min-h-11 flex-1 rounded-full border border-white/15 bg-white/5 px-5 text-sm text-white placeholder:text-white/35"
          />
          <MagneticButton type="submit" className="sm:w-auto">
            {busy ? "Reserving…" : "Reserve My Spot"}
          </MagneticButton>
        </form>
        {error ? <p className="mt-3 text-center text-sm text-rose-300">{error}</p> : null}
        {code ? (
          <p className="mt-3 text-center text-sm text-emerald-300">
            Spot reserved. Your code <span className="font-mono text-white">{code}</span> expires in 7 days.
          </p>
        ) : null}

        <div className="relative mt-10 flex flex-wrap items-center justify-center gap-5 text-xs text-white/80">
          {[
            [Lock, "Secure Payment"],
            [Shield, "30-Day Money Back"],
            [Zap, "Instant Access"],
            [Globe, "10,000+ Users"],
          ].map(([Icon, label], index) => (
            <motion.span
              key={label as string}
              initial={{ opacity: 0, y: 10 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.12 }}
              className="inline-flex items-center gap-1.5"
            >
              <Icon className="size-3.5 text-accent-cyan" /> {label as string}
            </motion.span>
          ))}
        </div>

        <div className="relative mx-auto mt-12 max-w-2xl space-y-3">
          {FAQ.map((item, index) => {
            const open = openFaq === index;
            return (
              <article key={item.q} className="overflow-hidden rounded-2xl border border-white/10 bg-black/40">
                <button
                  type="button"
                  className="focus-ring flex w-full items-center justify-between gap-4 px-5 py-4 text-left"
                  aria-expanded={open}
                  onClick={() => setOpenFaq(open ? null : index)}
                >
                  <span className="text-sm font-medium">{item.q}</span>
                  <ChevronDown className={cn("size-4 text-accent-cyan transition", open && "rotate-180")} />
                </button>
                <AnimatePresence initial={false}>
                  {open ? (
                    <motion.p
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      className="px-5 pb-4 text-sm text-mist"
                    >
                      {item.a}
                    </motion.p>
                  ) : null}
                </AnimatePresence>
              </article>
            );
          })}
        </div>

        <p className="relative mt-8 flex items-center justify-center gap-2 text-center text-xs text-white/45">
          <Sparkles className="size-3.5" /> Paid plans unlock in your account the moment checkout completes.
        </p>
      </div>
    </section>
  );
}
