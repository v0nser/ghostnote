import { AnimatePresence, MotionConfig, motion, useReducedMotion } from "motion/react";
import { useEffect, useState } from "react";

import { cast, site, type Who } from "../content/site";
import { useTypewriter } from "../lib/hooks";
import { Character } from "./cast/Character";
import { Subtitle } from "./cast/Subtitle";

const OPEN = site.hero.open;
const EASE = [0.22, 1, 0.36, 1] as const;

export function Hero() {
  return (
    <MotionConfig reducedMotion="user">
      <HeroInner />
    </MotionConfig>
  );
}

function HeroInner() {
  const reduced = useReducedMotion() ?? false;
  const [idx, setIdx] = useState(0);
  const line = OPEN[idx];
  const typed = useTypewriter(line.text, reduced, 22);

  useEffect(() => {
    if (!typed.done || idx >= OPEN.length - 1) return;
    const wait = reduced ? 1400 : 900;
    const t = window.setTimeout(() => setIdx((i) => i + 1), wait);
    return () => window.clearTimeout(t);
  }, [typed.done, idx, reduced]);

  const speaker = line.who as Who;
  const look = (who: Who): { x: number; y: number } => {
    if (who === speaker) return { x: 0, y: who === "coda" ? 0.35 : 0 };
    if (who === "coda") return { x: speaker === "engineer" ? -0.85 : 0.85, y: 0.15 };
    if (speaker === "coda") return { x: who === "engineer" ? 0.75 : -0.75, y: -0.05 };
    return { x: who === "engineer" ? 0.7 : -0.7, y: 0 };
  };

  return (
    <section className="wood relative flex min-h-[100svh] flex-col overflow-hidden pt-24">
      <div className="mx-auto flex w-full max-w-7xl flex-1 flex-col px-4 md:px-8">
        <div className="flex items-center justify-between gap-6">
          <p className="eyebrow">
            <span className="live-dot" /> {site.hero.kicker}
          </p>
          <a href="#story" className="font-mono text-[11px] uppercase tracking-[0.18em] text-ink-3 hover:text-ink">
            {site.hero.skip} ↓
          </a>
        </div>

        <div className="relative mt-4 flex min-h-0 flex-1 flex-col justify-center">
          <div className="mx-auto grid w-full max-w-4xl grid-cols-3 items-end gap-3 md:gap-10">
            <Actor who="engineer" speaker={speaker} typing={!typed.done} look={look("engineer")} />
            <Actor who="coda" speaker={speaker} typing={!typed.done} look={look("coda")} track />
            <Actor who="friend" speaker={speaker} typing={!typed.done} look={look("friend")} />
          </div>

          <div className="mt-5 min-h-[6.75rem]">
            <AnimatePresence mode="wait">
              <Subtitle key={`${idx}-${line.who}`} who={speaker} text={typed.shown} typing={!typed.done} />
            </AnimatePresence>
          </div>
        </div>

        <motion.div
          className="flex shrink-0 flex-col items-center gap-4 pb-10 pt-2 text-center"
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease: EASE, delay: 0.2 }}
        >
          <h1 className="font-display text-[clamp(2.1rem,6.2vw,4.6rem)] leading-[0.88] tracking-[-0.045em]">
            {site.hero.title}
            <span className="caption mt-1 block text-[0.72em] font-normal text-ink-2">{site.hero.italic}</span>
          </h1>
          <div className="flex flex-wrap items-center justify-center gap-3">
            <a href="#story" className="btn btn-ink">
              {site.hero.enter}
            </a>
            <a href={site.download.href} className="btn btn-ghost">
              {site.download.label}
            </a>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

function Actor({
  who,
  speaker,
  typing,
  look,
  track,
}: {
  who: Who;
  speaker: Who;
  typing: boolean;
  look: { x: number; y: number };
  track?: boolean;
}) {
  const on = who === speaker;
  return (
    <div className="flex flex-col items-center">
      <div className="relative aspect-square w-full max-w-[240px]">
        {on && <span className="spot pointer-events-none absolute inset-[10%] rounded-full" aria-hidden="true" />}
        <Character
          who={who}
          label={cast[who].label}
          mood={on && typing ? "curious" : who === "coda" ? "calm" : "tired"}
          speaking={on && typing}
          dim={!on}
          look={look}
          track={track}
        />
      </div>
      <p className="mt-1 font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">{cast[who].name}</p>
    </div>
  );
}
