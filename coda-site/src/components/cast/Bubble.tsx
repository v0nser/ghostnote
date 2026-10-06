import { motion } from "motion/react";

import type { Who } from "../../content/site";

interface Props {
  who: Who;
  name: string;
  text: string;
  active: boolean;
  typing: boolean;
  tail?: "center" | "none";
  className?: string;
}

export function Bubble({ who, name, text, active, typing, tail = "center", className }: Props) {
  const coda = who === "coda";
  return (
    <motion.div
      layout="position"
      initial={{ opacity: 0, y: 12, scale: 0.94 }}
      animate={{ opacity: active ? 1 : 0.42, y: 0, scale: active ? 1 : 0.97 }}
      exit={{ opacity: 0, y: -8, scale: 0.96, transition: { duration: 0.18 } }}
      transition={{ type: "spring", stiffness: 380, damping: 30 }}
      className={`relative w-full max-w-[290px] rounded-[20px] px-4 py-3 shadow-[0_18px_40px_-22px_rgba(22,20,15,.45)] ${
        coda ? "bg-ink text-paper" : "border border-ink/10 bg-card text-ink"
      } ${className ?? ""}`}
    >
      <span className={`block font-mono text-[10px] uppercase tracking-[0.2em] ${coda ? "text-accent" : "text-ink-3"}`}>
        {name}
      </span>
      <p className={`mt-1 text-[13.5px] leading-snug md:text-[15px] ${typing ? "caret" : ""}`} aria-live={active ? "polite" : "off"}>
        {text}
      </p>
      {tail === "center" && (
        <span
          aria-hidden="true"
          className={`absolute -bottom-[6px] left-1/2 size-3 -translate-x-1/2 rotate-45 ${
            coda ? "bg-ink" : "border-b border-r border-ink/10 bg-card"
          }`}
        />
      )}
    </motion.div>
  );
}
