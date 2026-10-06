"use client";

import { useState } from "react";

import { MagneticButton } from "@/components/magnetic-button";
import { AppWindow } from "@/components/mockups/app-window";

export function Hero() {
  const [stealth, setStealth] = useState(false);

  return (
    <section className="relative overflow-hidden px-5 pb-20 pt-16 md:pt-24">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_0%,rgba(200,200,204,0.07),transparent_42%)]" />
      <div className="relative mx-auto grid max-w-6xl items-center gap-12 lg:grid-cols-[1.05fr_0.95fr]">
        <div>
          <p className="mb-4 text-xs uppercase tracking-[0.28em] text-mist">GhostNote</p>
          <h1 className="max-w-xl text-4xl font-semibold leading-[1.08] text-white sm:text-5xl lg:text-6xl">
            A local AI agent.
            <span className="block text-mist">Invisible in meetings.</span>
          </h1>
          <p className="mt-6 max-w-lg text-base leading-relaxed text-mist">
            Plan, research, draft, and remember on your machine. The meeting
            copilot stays hidden on screen share. Nothing leaves unless you
            connect a cloud model.
          </p>
          <div className="mt-8 flex flex-wrap items-center gap-3">
            <MagneticButton href="#cta">Download Free</MagneticButton>
            <MagneticButton href="#capabilities" variant="secondary">
              See capabilities
            </MagneticButton>
          </div>
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
