"use client";

import { useState } from "react";

import { CountdownTimer } from "@/components/countdown-timer";
import { MagneticButton } from "@/components/magnetic-button";
import { AppWindow } from "@/components/mockups/app-window";
import { useCountdown } from "@/hooks/use-countdown";

export function Hero() {
  const [stealth, setStealth] = useState(false);
  const { expired } = useCountdown();

  return (
    <section className="relative overflow-hidden px-5 pb-20 pt-16 md:pt-24">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_0%,rgba(200,200,204,0.07),transparent_42%)]" />
      <div className="relative mx-auto grid max-w-6xl items-center gap-12 lg:grid-cols-[1.05fr_0.95fr]">
        <div>
          <p className="mb-4 text-xs uppercase tracking-[0.28em] text-mist">GhostNote</p>
          <h1 className="max-w-xl text-4xl font-semibold leading-[1.08] text-white sm:text-5xl lg:text-6xl">
            Invisible intelligence.
            <span className="block text-mist">Unforgettable meetings.</span>
          </h1>
          <p className="mt-6 max-w-lg text-base leading-relaxed text-mist">
            The only AI meeting assistant that stays hidden during screen shares.
          </p>
          <div className="mt-8 flex flex-wrap items-center gap-3">
            <MagneticButton href="#cta">Download Free</MagneticButton>
            <MagneticButton href="#demo" variant="secondary">
              See How It Works
            </MagneticButton>
          </div>
          <a
            href="#early-bird"
            className="focus-ring mt-5 inline-flex flex-wrap items-center gap-2 rounded-full border border-accent-cyan/30 bg-white/5 px-4 py-2 text-sm text-white/80"
          >
            🎉 Early Bird: 50% OFF
            {expired ? <span>ended</span> : <CountdownTimer compact className="text-accent-cyan" />}
          </a>
        </div>
        <AppWindow
          stealth={stealth}
          onToggle={() => setStealth((value) => !value)}
          showTranscript
          showAnswer={!stealth}
        />
      </div>
    </section>
  );
}
