"use client";

import { CountdownTimer } from "@/components/countdown-timer";
import { useCountdown } from "@/hooks/use-countdown";

export function EarlyBirdBanner() {
  const { expired } = useCountdown();

  if (expired) return null;

  return (
    <a
      href="#early-bird"
      className="focus-ring flex items-center justify-center gap-3 border-b border-accent-cyan/20 bg-black/70 px-4 py-2 text-xs text-white/80 backdrop-blur md:text-sm"
    >
      <span className="relative flex size-2">
        <span className="absolute inline-flex size-full animate-ping rounded-full bg-rose-400 opacity-75" />
        <span className="relative inline-flex size-2 rounded-full bg-rose-400" />
      </span>
      <span>Early Bird: 50% OFF lifetime access</span>
      <CountdownTimer compact className="text-accent-cyan" />
    </a>
  );
}
