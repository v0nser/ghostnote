"use client";

import { useState } from "react";
import { Check } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";

import { MagneticButton } from "@/components/magnetic-button";
import { cn } from "@/lib/utils";

const TIERS = [
  {
    name: "Community",
    monthly: 0,
    yearly: 0,
    blurb: "Local copilot. Full stealth. Yours.",
    cta: "Download Free",
    href: "/downloads/GhostNote.dmg",
    features: ["Local Whisper + Ollama", "Stealth mode", "Interview answers", "Meeting summary"],
  },
  {
    name: "Pro",
    monthly: 15,
    yearly: 12,
    popular: true,
    blurb: "Encrypted sync and faster models.",
    cta: "Claim 50% Early Bird",
    href: "/checkout?plan=pro",
    features: [
      "Everything in Community",
      "Encrypted cloud sync",
      "Advanced AI models",
      "Priority support",
    ],
  },
  {
    name: "Team",
    monthly: 30,
    yearly: 24,
    blurb: "Shared playbooks for interview pods.",
    cta: "Get Team Discount",
    href: "/checkout?plan=team",
    features: ["Everything in Pro", "Shared prompts", "Admin controls", "SSO"],
  },
];

export function Pricing() {
  const [annual, setAnnual] = useState(false);
  const [open, setOpen] = useState(false);

  return (
    <section id="pricing" className="px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-semibold md:text-4xl">Choose Your Level of Stealth</h2>
        <div className="mx-auto mt-8 flex w-fit items-center gap-3 rounded-full border border-white/10 bg-white/5 p-1">
          <button
            type="button"
            onClick={() => setAnnual(false)}
            className={cn("rounded-full px-4 py-1.5 text-sm", !annual ? "bg-white text-ink" : "text-mist")}
          >
            Monthly
          </button>
          <button
            type="button"
            onClick={() => setAnnual(true)}
            className={cn("rounded-full px-4 py-1.5 text-sm", annual ? "bg-white text-ink" : "text-mist")}
          >
            Annual (Save 20%)
          </button>
        </div>

        <div className="mt-12 grid gap-5 lg:grid-cols-3">
          {TIERS.map((tier) => {
            const price = annual ? tier.yearly : tier.monthly;
            return (
              <article
                key={tier.name}
                className={cn(
                  "relative rounded-3xl border p-6 transition hover:-translate-y-1",
                  tier.popular
                    ? "border-cyan/40 bg-cyan/5 shadow-glow"
                    : "border-white/10 bg-charcoal",
                )}
              >
                {tier.popular ? (
                  <p className="absolute -top-3 left-6 animate-pulse rounded-full bg-cyan-purple px-3 py-0.5 text-[10px] font-semibold text-ink">
                    Most Popular
                  </p>
                ) : null}
                <h3 className="text-xl font-semibold">{tier.name}</h3>
                <p className="mt-1 text-sm text-mist">{tier.blurb}</p>
                <p className="mt-6 flex items-end gap-1">
                  <span className="text-4xl font-semibold">${price}</span>
                  <span className="mb-1 text-sm text-mist">
                    {tier.name === "Team" ? "/user/month" : "/month"}
                  </span>
                </p>
                {annual && tier.monthly > 0 ? (
                  <p className="mt-1 text-xs text-cyan">
                    ${tier.yearly * 12}/year · save ${((tier.monthly - tier.yearly) * 12)}
                  </p>
                ) : null}
                <ul className="mt-6 space-y-2 text-sm">
                  {tier.features.map((feature) => (
                    <li key={feature} className="flex gap-2 text-white">
                      <Check className="mt-0.5 size-4 text-cyan" /> {feature}
                    </li>
                  ))}
                </ul>
                <MagneticButton
                  href={tier.href}
                  variant={tier.popular ? "primary" : "secondary"}
                  className="mt-8 w-full"
                  download={tier.href.endsWith(".dmg") ? "GhostNote.dmg" : undefined}
                >
                  {tier.cta}
                </MagneticButton>
              </article>
            );
          })}
        </div>

        <button
          type="button"
          onClick={() => setOpen((value) => !value)}
          className="focus-ring mx-auto mt-10 block text-sm text-mist underline-offset-4 hover:text-white hover:underline"
        >
          {open ? "Hide comparison" : "Compare all features"}
        </button>
        <AnimatePresence>
          {open ? (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              className="overflow-hidden"
            >
              <table className="mt-6 w-full text-left text-sm">
                <thead className="text-mist">
                  <tr>
                    <th className="py-2 font-medium">Feature</th>
                    <th>Community</th>
                    <th>Pro</th>
                    <th>Team</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    ["Local AI", "Yes", "Yes", "Yes"],
                    ["Stealth", "Yes", "Yes", "Yes"],
                    ["Cloud sync", "—", "Yes", "Yes"],
                    ["SSO", "—", "—", "Yes"],
                  ].map((row) => (
                    <tr key={row[0]} className="border-t border-white/8 hover:bg-white/5">
                      {row.map((cell) => (
                        <td key={cell} className="py-2.5">
                          {cell}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </motion.div>
          ) : null}
        </AnimatePresence>
      </div>
    </section>
  );
}
