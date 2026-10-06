import { AnimatePresence, LayoutGroup, MotionConfig, motion, useReducedMotion } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { cast, site, type Chapter, type PropId, type Who } from "../content/site";
import { useMediaQuery, useTypewriter } from "../lib/hooks";
import { Character } from "./cast/Character";
import { PROP_ORDER, PropCard } from "./cast/Props";
import { Subtitle } from "./cast/Subtitle";

const CHAPTERS: Chapter[] = site.story.chapters;
const SEATS: Who[] = ["engineer", "coda", "friend"];
const SEAT_X: Record<Who, number> = { engineer: -1, coda: 0, friend: 1 };

export function Stage() {
  return (
    <MotionConfig reducedMotion="user">
      <StageInner />
    </MotionConfig>
  );
}

function StageInner() {
  const section = useRef<HTMLElement>(null);
  const bar = useRef<HTMLDivElement>(null);
  const [chapter, setChapter] = useState(0);
  const reduced = useReducedMotion() ?? false;
  const wide = useMediaQuery("(min-width: 900px)");

  useEffect(() => {
    let frame = 0;
    const update = () => {
      frame = 0;
      const el = section.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const total = Math.max(1, rect.height - window.innerHeight);
      const p = Math.min(0.9999, Math.max(0, -rect.top / total));
      if (bar.current) bar.current.style.transform = `scaleX(${p})`;
      setChapter(Math.floor(p * CHAPTERS.length));
    };
    const onScroll = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    update();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll);
    return () => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
      cancelAnimationFrame(frame);
    };
  }, []);

  const ch = CHAPTERS[chapter];
  const [cue, setCue] = useState({ ch: 0, idx: 0 });
  const idx = cue.ch === chapter ? cue.idx : 0;
  const line = ch.lines[idx];
  const typed = useTypewriter(line.text, reduced, 20);

  useEffect(() => {
    if (!typed.done || idx >= ch.lines.length - 1) return;
    const timer = window.setTimeout(() => setCue({ ch: chapter, idx: idx + 1 }), reduced ? 1500 : 950);
    return () => window.clearTimeout(timer);
  }, [typed.done, idx, chapter, ch.lines.length, reduced]);

  const resolved = new Set<PropId>();
  if (ch.props === "dock") PROP_ORDER.forEach((id) => resolved.add(id));
  if (ch.props === "resolved") ch.lines.slice(0, idx + 1).forEach((l) => l.resolves?.forEach((r) => resolved.add(r)));
  const visible = wide ? PROP_ORDER : PROP_ORDER.slice(0, 3);

  const speaker = line.who;
  const speaking = (who: Who) => who === speaker && !typed.done;
  const lookFor = (who: Who) => {
    if (who === "coda") return speaker === "coda" ? { x: 0, y: 0.35 } : { x: SEAT_X[speaker] * 0.9, y: 0.2 };
    const target = speaker === who ? (ch.coda ? "coda" : who === "engineer" ? "friend" : "engineer") : speaker;
    return { x: Math.sign(SEAT_X[target] - SEAT_X[who]) * 0.85, y: 0 };
  };

  const goTo = (i: number) => {
    const el = section.current;
    if (!el) return;
    const top = el.getBoundingClientRect().top + window.scrollY;
    const total = el.offsetHeight - window.innerHeight;
    window.scrollTo({ top: top + ((i + 0.35) / CHAPTERS.length) * total, behavior: reduced ? "auto" : "smooth" });
  };

  return (
    <section ref={section} id="story" aria-label="The story" style={{ height: `${CHAPTERS.length * 95}svh` }} className="relative bg-stage text-paper">
      <div className="sticky top-0 h-[100svh] overflow-hidden">
        <div className="night-grid absolute inset-0 opacity-70" aria-hidden="true" />
        <div className="relative mx-auto flex h-full max-w-7xl flex-col px-4 pb-5 pt-20 md:px-8 md:pt-24">
          <div className="flex items-start justify-between gap-6">
            <AnimatePresence mode="wait">
              <motion.div
                key={ch.id}
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                transition={{ duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
              >
                <p className="eyebrow text-paper/45">
                  Scene {ch.num} / 0{CHAPTERS.length}
                </p>
                <h2 className="mt-1 font-display text-[clamp(2.2rem,5.5vw,4.4rem)] leading-[0.9] tracking-[-0.04em]">{ch.title}</h2>
                <p className="caption mt-2 max-w-md text-sm text-paper/55 md:text-base">{ch.caption}</p>
              </motion.div>
            </AnimatePresence>

            <nav className="hidden shrink-0 gap-1 rounded-full border border-paper/12 bg-night/50 p-1 backdrop-blur md:flex" aria-label="Scenes">
              {CHAPTERS.map((c, i) => (
                <button
                  key={c.id}
                  type="button"
                  onClick={() => goTo(i)}
                  aria-current={i === chapter ? "step" : undefined}
                  className="relative rounded-full px-3.5 py-1.5 font-mono text-[11px] uppercase tracking-[0.14em] text-paper/60"
                >
                  {i === chapter && (
                    <motion.span layoutId="chapter-pill" className="absolute inset-0 rounded-full bg-paper" transition={{ type: "spring", stiffness: 380, damping: 32 }} />
                  )}
                  <span className={`relative ${i === chapter ? "text-night" : ""}`}>{c.title.replace("The ", "")}</span>
                </button>
              ))}
            </nav>
          </div>

          <div className="mt-4 grid min-h-0 flex-1 gap-4 md:grid-cols-[minmax(0,0.9fr)_minmax(0,1.2fr)]">
            <div className="panel relative hidden overflow-hidden bg-card text-ink md:block">
              <p className="absolute left-4 top-3 z-10 font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">close-up</p>
              <AnimatePresence mode="wait">
                <motion.div
                  key={speaker}
                  className="absolute inset-0"
                  initial={{ opacity: 0, scale: 1.06 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.98 }}
                  transition={{ duration: 0.45, ease: [0.22, 1, 0.36, 1] }}
                >
                  <span className="spot absolute inset-[10%] top-[18%]" aria-hidden="true" />
                  <div className="absolute inset-x-[6%] bottom-[-8%] top-[8%]">
                    <Character
                      who={speaker}
                      close
                      label={cast[speaker].label}
                      mood={ch.moods[speaker]}
                      speaking={speaking(speaker)}
                      look={lookFor(speaker)}
                    />
                  </div>
                </motion.div>
              </AnimatePresence>
            </div>

            <div className="relative min-h-0">
              <LayoutGroup>
                <div className="relative h-[38%] min-h-[110px] md:h-[42%]">
                  {ch.props === "settling" && <ArrivalTrail />}
                  {ch.props !== "dock" && (
                    <div className="absolute inset-0">
                      {visible
                        .filter((id) => !resolved.has(id))
                        .map((id, i) => (
                          <PropCard key={id} id={id} index={i} mode={ch.props === "chaos" ? "chaos" : "settling"} />
                        ))}
                    </div>
                  )}
                  {ch.props === "resolved" && (
                    <div className="relative z-10 grid grid-cols-2 content-start gap-2 md:grid-cols-3">
                      {visible
                        .filter((id) => resolved.has(id))
                        .map((id) => (
                          <PropCard key={id} id={id} mode="resolved" />
                        ))}
                    </div>
                  )}
                  {ch.props === "dock" && (
                    <div className="flex h-full flex-wrap content-center items-center justify-center gap-2">
                      {visible.map((id, i) => (
                        <PropCard key={id} id={id} index={i} mode="dock" />
                      ))}
                    </div>
                  )}
                </div>
              </LayoutGroup>

              <div className="mt-auto grid grid-cols-3 items-end gap-2 md:gap-6">
                {SEATS.map((who) => {
                  const present = who !== "coda" || ch.coda;
                  return (
                    <div key={who} className="flex flex-col items-center">
                      <div className="relative aspect-square w-full max-w-[160px] overflow-hidden md:max-w-[200px]">
                        <AnimatePresence>
                          {present && (
                            <motion.div
                              key={who}
                              className="absolute inset-0"
                              initial={who === "coda" ? { opacity: 0, y: -28, scale: 0.5 } : { opacity: 0, y: 24 }}
                              animate={{ opacity: 1, x: 0, y: 0, scale: 1 }}
                              exit={{ opacity: 0, scale: 0.6 }}
                              transition={{ type: "spring", stiffness: 140, damping: 18 }}
                            >
                              {speaking(who) && <span className="spot absolute inset-[12%] rounded-full" aria-hidden="true" />}
                              <Character
                                who={who}
                                label={cast[who].label}
                                mood={ch.moods[who]}
                                speaking={speaking(who)}
                                dim={!speaking(who) && speaker !== who}
                                look={lookFor(who)}
                              />
                            </motion.div>
                          )}
                        </AnimatePresence>
                      </div>
                      <p className={`mt-1 font-mono text-[10px] uppercase tracking-[0.18em] ${present ? "text-paper/45" : "text-transparent"}`}>
                        {cast[who].name}
                      </p>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          <div className="mt-3 min-h-[6.5rem]">
            <AnimatePresence mode="wait">
              <Subtitle key={`${ch.id}-${idx}`} who={speaker} text={typed.shown} typing={!typed.done} />
            </AnimatePresence>
          </div>
        </div>

        <div className="absolute inset-x-0 bottom-0 h-[2px] bg-paper/10" aria-hidden="true">
          <div ref={bar} className="h-full origin-left bg-accent" style={{ transform: "scaleX(0)" }} />
        </div>
      </div>
    </section>
  );
}

function ArrivalTrail() {
  return (
    <svg className="pointer-events-none absolute inset-0 h-full w-full" viewBox="0 0 1000 400" preserveAspectRatio="none" aria-hidden="true">
      <motion.path
        d="M1010 30 C 820 40, 760 180, 640 250 S 520 360, 500 410"
        fill="none"
        stroke="#d4521a"
        strokeWidth="2.5"
        strokeLinecap="round"
        vectorEffect="non-scaling-stroke"
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: [0, 1, 1, 0.25] }}
        transition={{ duration: 1.5, ease: [0.22, 1, 0.36, 1] }}
      />
    </svg>
  );
}
