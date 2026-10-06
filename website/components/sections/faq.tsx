"use client";

import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown } from "lucide-react";

const QA = [
  {
    q: "Is my data really private?",
    a: "Yes by default. Audio, transcripts, memory, tasks, and audit logs stay on the machine. Cloud models stay off until you configure them. Secrets are never written into logs.",
  },
  {
    q: "Does it work on Windows?",
    a: "Yes. Use GhostNote-Setup.exe, open Setup, download the speech model, and install Ollama. Participant audio uses WASAPI loopback — no virtual cable. Building from source: scripts/setup-whisper.ps1. Details in docs/WINDOWS_SETUP.md.",
  },
  {
    q: "What can the agent actually do?",
    a: "Plan multi-step work, search the web, fetch pages, read/write its workspace, run short sandboxed code, remember facts, and schedule follow-ups. Email send, deletes, and computer control require approval or stay denied.",
  },
  {
    q: "Will it send email or submit applications by itself?",
    a: "No. Ghost Sentinel blocks silent send/submit/purchase. Those actions pause for Approve once / Deny.",
  },
  {
    q: "How do I make replies faster?",
    a: "Pull a small model: ollama pull llama3.2:3b or qwen2.5:3b. Ghost Note prefers those over 8B+ models. Live answers only send the last 45 seconds of transcript.",
  },
  {
    q: "Can I still use it only as a meeting copilot?",
    a: "Yes. The Meeting tab is the original product. The Agent tab is additive. Stealth mode is unchanged.",
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
