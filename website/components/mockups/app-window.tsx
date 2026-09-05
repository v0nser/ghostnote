"use client";

import { motion } from "framer-motion";

import { cn } from "@/lib/utils";

export function AppWindow({
  stealth,
  onToggle,
  showTranscript = true,
  showAnswer = true,
  compact = false,
}: {
  stealth: boolean;
  onToggle: () => void;
  showTranscript?: boolean;
  showAnswer?: boolean;
  compact?: boolean;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.94, y: 16 }}
      animate={{ opacity: stealth ? 0.12 : 1, scale: 1, y: 0 }}
      transition={{ duration: 0.55 }}
      className={cn(
        "glass overflow-hidden rounded-2xl shadow-glow",
        compact ? "min-h-[280px]" : "min-h-[360px]",
      )}
    >
      <div className="flex items-center justify-between border-b border-white/8 px-4 py-2.5">
        <div className="flex items-center gap-1.5" aria-hidden>
          <span className="size-2.5 rounded-full bg-[#ff5f57]" />
          <span className="size-2.5 rounded-full bg-[#febc2e]" />
          <span className="size-2.5 rounded-full bg-[#28c840]" />
        </div>
        <p className="text-[11px] tracking-wide text-mist">GhostNote</p>
        <button
          type="button"
          onClick={onToggle}
          className="focus-ring flex items-center gap-2 rounded-full px-2 py-1 text-[11px] text-mist"
          aria-pressed={stealth}
          aria-label="Toggle stealth mode"
        >
          Stealth
          <span
            className={cn(
              "relative h-5 w-9 rounded-full transition-colors",
              stealth ? "bg-cyan" : "bg-white/15",
            )}
          >
            <span
              className={cn(
                "absolute top-0.5 size-4 rounded-full bg-white transition-transform",
                stealth ? "translate-x-4" : "translate-x-0.5",
              )}
            />
          </span>
        </button>
      </div>

      <div className={cn("grid gap-0 md:grid-cols-[1.1fr_0.9fr]", compact && "md:grid-cols-1")}>
        <div className="space-y-2.5 p-4">
          <p className="text-[10px] uppercase tracking-[0.18em] text-mist">Them</p>
          {showTranscript ? (
            <TypeLine text="Can you walk me through a closure in JavaScript?" />
          ) : (
            <p className="text-sm text-white/30">Waiting for the interviewer…</p>
          )}
        </div>
        <div className="border-t border-white/8 p-4 md:border-l md:border-t-0">
          <p className="text-[10px] uppercase tracking-[0.18em] text-mist">Say this</p>
          {showAnswer ? (
            <motion.div
              initial={{ y: 18, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              className="mt-2 rounded-xl bg-white/5 p-3"
            >
              <TypeLine text="A closure is a function that remembers variables from the scope where it was created. I use them for private state in hooks." />
            </motion.div>
          ) : (
            <p className="mt-3 animate-pulse text-sm text-cyan/80">Processing…</p>
          )}
        </div>
      </div>
    </motion.div>
  );
}

function TypeLine({ text }: { text: string }) {
  return (
    <motion.p className="text-sm leading-relaxed text-white" aria-label={text}>
      {text.split("").map((char, index) => (
        <motion.span
          key={`${char}-${index}`}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: index * 0.012 }}
        >
          {char}
        </motion.span>
      ))}
    </motion.p>
  );
}
