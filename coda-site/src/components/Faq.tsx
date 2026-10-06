import { AnimatePresence, MotionConfig, motion } from "motion/react";
import { useState } from "react";

import { site } from "../content/site";

export function Faq() {
  const [open, setOpen] = useState<number | null>(0);
  return (
    <MotionConfig reducedMotion="user">
      <section id="faq" className="mx-auto grid max-w-7xl gap-10 px-4 py-24 md:grid-cols-[1fr_1.6fr] md:px-8 md:py-32">
        <div>
          <p className="eyebrow">{site.faq.eyebrow}</p>
          <h2 className="mt-4 font-display text-[clamp(2.6rem,6vw,5rem)] leading-[0.9] tracking-[-0.04em]">{site.faq.title}</h2>
        </div>
        <div className="border-t border-ink/10">
          {site.faq.items.map((item, i) => {
            const expanded = open === i;
            return (
              <div key={item.q} className="border-b border-ink/10">
                <h3>
                  <button
                    type="button"
                    aria-expanded={expanded}
                    onClick={() => setOpen(expanded ? null : i)}
                    className="flex w-full items-center justify-between gap-6 py-6 text-left text-lg md:text-xl"
                  >
                    {item.q}
                    <span
                      aria-hidden="true"
                      className={`grid size-8 shrink-0 place-items-center rounded-full border border-ink/15 text-lg transition-transform duration-300 ${expanded ? "rotate-45 bg-ink text-paper" : ""}`}
                    >
                      +
                    </span>
                  </button>
                </h3>
                <AnimatePresence initial={false}>
                  {expanded && (
                    <motion.div
                      key="a"
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.35, ease: [0.2, 0.8, 0.2, 1] }}
                      className="overflow-hidden"
                    >
                      <p className="max-w-2xl pb-6 pr-12 text-ink-2">{item.a}</p>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            );
          })}
        </div>
      </section>
    </MotionConfig>
  );
}
