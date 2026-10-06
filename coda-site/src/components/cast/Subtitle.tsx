import { motion } from "motion/react";

import { cast, type Who } from "../../content/site";

interface Props {
  who: Who;
  text: string;
  typing?: boolean;
  className?: string;
}

export function Subtitle({ who, text, typing, className }: Props) {
  const coda = who === "coda";
  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 8 }}
      transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
      className={`mx-auto w-full max-w-3xl rounded-[18px] px-5 py-3.5 text-center ${
        coda ? "bg-ink text-paper" : "border-2 border-ink bg-card text-ink"
      } ${className ?? ""}`}
    >
      <span className={`block font-mono text-[10px] uppercase tracking-[0.22em] ${coda ? "text-accent" : "text-ink-3"}`}>
        {cast[who].name}
      </span>
      <p className={`caption mt-1 text-[1.15rem] leading-snug md:text-[1.45rem] ${typing ? "caret" : ""}`} aria-live="polite">
        {text}
        {typing ? "\u00a0" : ""}
      </p>
    </motion.div>
  );
}
