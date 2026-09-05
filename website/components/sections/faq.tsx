"use client";

import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown } from "lucide-react";

const QA = [
  {
    q: "Is my data really 100% private?",
    a: "Yes. Audio never leaves the machine. Whisper and Ollama run locally. We do not see the meeting.",
  },
  {
    q: "Does it work on Mac, Windows, and Linux?",
    a: "macOS is first-class today, including system-audio capture. Windows and Linux follow the same Tauri shell.",
  },
  {
    q: "What AI models does it support?",
    a: "Anything Ollama can serve. We ship tuned for llama3.1:latest. Mistral and Qwen work the same day you pull them.",
  },
  {
    q: "Can I use it during client meetings?",
    a: "That is the point. Stealth excludes the window from capture so a screen share never shows GhostNote.",
  },
  {
    q: "How does the stealth mode actually work?",
    a: "The OS capture-exclusion API keeps the GhostNote window out of ScreenCaptureKit / similar taps. Their share sees your meeting app. It does not see us.",
  },
];

export function Faq() {
  const [open, setOpen] = useState<number | null>(0);

  return (
    <section id="faq" className="px-5 py-24">
      <div className="mx-auto max-w-3xl">
        <h2 className="text-center text-3xl font-semibold md:text-4xl">
          Questions? We&apos;ve Got Answers.
        </h2>
        <div className="mt-10 space-y-3">
          {QA.map((item, index) => {
            const expanded = open === index;
            return (
              <article key={item.q} className="overflow-hidden rounded-2xl border border-white/10 bg-charcoal">
                <button
                  type="button"
                  className="focus-ring flex w-full items-center justify-between gap-4 px-5 py-4 text-left"
                  aria-expanded={expanded}
                  onClick={() => setOpen(expanded ? null : index)}
                >
                  <span className="text-sm font-medium md:text-base">{item.q}</span>
                  <ChevronDown
                    className={`size-4 shrink-0 text-cyan transition-transform ${expanded ? "rotate-180" : ""}`}
                  />
                </button>
                <AnimatePresence initial={false}>
                  {expanded ? (
                    <motion.p
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      className="px-5 pb-5 text-sm leading-relaxed text-mist"
                    >
                      {item.a}
                    </motion.p>
                  ) : null}
                </AnimatePresence>
              </article>
            );
          })}
        </div>
      </div>
    </section>
  );
}
