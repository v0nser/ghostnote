"use client";

import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";

import { useCountdown, type TimeLeft } from "@/hooks/use-countdown";
import { cn } from "@/lib/utils";

const UNITS: { key: keyof Pick<TimeLeft, "days" | "hours" | "minutes" | "seconds">; label: string }[] = [
  { key: "days", label: "Days" },
  { key: "hours", label: "Hours" },
  { key: "minutes", label: "Minutes" },
  { key: "seconds", label: "Seconds" },
];

function FlipDigit({ value }: { value: number }) {
  const display = String(value).padStart(2, "0");
  return (
    <span className="relative inline-block h-[1.15em] overflow-hidden">
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.span
          key={display}
          initial={{ rotateX: -80, opacity: 0, y: "-40%" }}
          animate={{ rotateX: 0, opacity: 1, y: "0%" }}
          exit={{ rotateX: 80, opacity: 0, y: "40%" }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="block origin-center"
          style={{ backfaceVisibility: "hidden" }}
        >
          {display}
        </motion.span>
      </AnimatePresence>
    </span>
  );
}

export function CountdownTimer({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  const time = useCountdown();
  const [live, setLive] = useState("");
  const announced = useRef("");

  useEffect(() => {
    if (time.expired) {
      setLive("Early Bird offer expired.");
      return;
    }
    const next = `${time.days} days, ${time.hours} hours, ${time.minutes} minutes remaining`;
    if (time.seconds === 0 && next !== announced.current) {
      announced.current = next;
      setLive(next);
    }
  }, [time]);

  if (compact) {
    const label = time.expired
      ? "Offer ended"
      : `${String(time.days).padStart(2, "0")}:${String(time.hours).padStart(2, "0")}:${String(time.minutes).padStart(2, "0")}:${String(time.seconds).padStart(2, "0")}`;
    return (
      <span className={cn("font-mono tabular-nums text-white", className)}>
        {label}
        <span className="sr-only" aria-live="polite">
          {live}
        </span>
      </span>
    );
  }

  return (
    <div
      className={cn("grid grid-cols-2 gap-3 sm:grid-cols-4 sm:gap-4", className)}
      role="timer"
      aria-label="Early Bird countdown"
    >
      {UNITS.map((unit) => (
        <motion.div
          key={unit.key}
          className={cn(
            "countdown-box flex flex-col items-center justify-center rounded-2xl border border-accent-cyan/30 bg-black/50 backdrop-blur-md",
            compact ? "size-16" : "h-20 w-full sm:h-[120px] sm:w-[120px]",
            time.days < 1 && !time.expired && "animate-urgent-shake",
          )}
          animate={{ boxShadow: ["0 0 0 rgba(34,211,238,0)", "0 0 18px rgba(34,211,238,0.28)", "0 0 0 rgba(34,211,238,0)"] }}
          transition={{ duration: 1, repeat: Infinity }}
        >
          <p className="font-mono text-3xl font-bold text-white sm:text-5xl">
            <FlipDigit value={time[unit.key]} />
          </p>
          <p className="mt-1 text-[11px] uppercase tracking-[0.18em] text-accent-cyan">{unit.label}</p>
        </motion.div>
      ))}
      <span className="sr-only" aria-live="polite">
        {live}
      </span>
    </div>
  );
}
