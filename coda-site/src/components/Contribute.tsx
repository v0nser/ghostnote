import { AnimatePresence, MotionConfig, motion, useInView } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { site } from "../content/site";
import { useTypewriter } from "../lib/hooks";
import { Robot } from "./cast/Robot";

const c = site.contribute;
const REPO = `https://github.com/${c.repo}`;
const EASE = [0.2, 0.8, 0.2, 1] as const;

type Live = { stars: number; forks: number; issues: number };

function useRepoStats(enabled: boolean) {
  const [live, setLive] = useState<Live | null>(null);
  useEffect(() => {
    if (!enabled) return;
    const ctrl = new AbortController();
    fetch(`https://api.github.com/repos/${c.repo}`, { signal: ctrl.signal })
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => {
        if (d && typeof d.stargazers_count === "number") {
          setLive({ stars: d.stargazers_count, forks: d.forks_count, issues: d.open_issues_count });
        }
      })
      .catch(() => {});
    return () => ctrl.abort();
  }, [enabled]);
  return live;
}

export function Contribute() {
  const ref = useRef<HTMLElement>(null);
  const inView = useInView(ref, { amount: 0.3 });
  const seen = useInView(ref, { once: true, margin: "200px" });
  const live = useRepoStats(seen);

  const [line, setLine] = useState(0);
  const [poked, setPoked] = useState(false);
  const text = c.pokes[line];
  const { shown, done } = useTypewriter(text, !inView, 26);

  useEffect(() => {
    if (!inView || !done) return;
    const t = window.setTimeout(() => setLine((l) => (l + 1) % c.pokes.length), 3600);
    return () => window.clearTimeout(t);
  }, [inView, done, line]);

  const poke = () => {
    setPoked(true);
    window.setTimeout(() => setPoked(false), 900);
    setLine((l) => (l + 1) % c.pokes.length);
  };

  return (
    <MotionConfig reducedMotion="user">
      <section
        id="build"
        ref={ref}
        className="relative scroll-mt-24 overflow-hidden bg-stage px-4 py-24 text-paper md:px-8 md:py-32"
      >
        <div className="night-grid pointer-events-none absolute inset-0" aria-hidden="true" />

        <div className="relative mx-auto grid max-w-7xl items-center gap-12 lg:grid-cols-[0.9fr_1.1fr]">
          {/* Patch, the robot */}
          <div className="relative mx-auto w-full max-w-md">
            <div className="spot absolute inset-x-8 top-10 h-64" aria-hidden="true" />
            <div className="relative" aria-live="polite">
              <AnimatePresence mode="wait">
                <motion.div
                  key={line}
                  initial={{ opacity: 0, y: 8, scale: 0.96 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.3, ease: EASE }}
                  className="panel relative mx-auto mb-4 min-h-[5.5rem] max-w-sm px-5 py-4 text-ink"
                >
                  <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">{c.botName}</p>
                  <p className="caption mt-1 text-lg leading-snug">
                    {shown}
                    {!done && <span className="caret" aria-hidden="true" />}
                  </p>
                  <span
                    aria-hidden="true"
                    className="absolute -bottom-[9px] left-1/2 size-4 -translate-x-1/2 rotate-45 border-b-2 border-r-2 border-ink bg-card"
                  />
                </motion.div>
              </AnimatePresence>
            </div>
            <button
              type="button"
              onClick={poke}
              aria-label={`Poke ${c.botName} for another line`}
              className="group relative mx-auto block h-72 w-64 cursor-pointer md:h-96 md:w-80"
            >
              <motion.div
                className="h-full w-full"
                animate={poked ? { rotate: [0, -6, 5, -3, 0], y: [0, -10, 0] } : {}}
                transition={{ duration: 0.7, ease: EASE }}
              >
                <Robot label={c.botLabel} speaking={inView && !done} happy={poked} track={inView} />
              </motion.div>
              <span className="pointer-events-none absolute -bottom-2 left-1/2 -translate-x-1/2 font-mono text-[10px] uppercase tracking-[0.2em] text-paper/45 transition-colors group-hover:text-accent">
                poke me
              </span>
            </button>
          </div>

          {/* The ask */}
          <div>
            <p className="eyebrow !text-paper/55">
              <span className="live-dot" aria-hidden="true" />
              {c.eyebrow}
            </p>
            <h2 className="mt-4 font-display text-[clamp(2.6rem,6vw,5.2rem)] leading-[0.9] tracking-[-0.04em]">
              {c.title}{" "}
              <span className="caption font-normal text-accent">{c.titleItalic}</span>
            </h2>
            <p className="mt-6 max-w-xl text-lg text-paper/75">{c.intro}</p>

            <div className="mt-8 flex flex-wrap items-center gap-3">
              <a className="btn btn-paper" href={REPO} target="_blank" rel="noreferrer">
                <GitMark /> {c.primary}
              </a>
              <a
                className="btn border border-paper/25 text-paper hover:bg-paper/10"
                href={`${REPO}/issues/new`}
                target="_blank"
                rel="noreferrer"
              >
                {c.secondary}
              </a>
            </div>

            <AnimatePresence>
              {live && (
                <motion.dl
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, ease: EASE }}
                  className="mt-6 flex flex-wrap gap-x-6 gap-y-2 font-mono text-xs text-paper/60"
                >
                  {(["stars", "forks", "issues"] as const).map((k) => (
                    <div key={k} className="flex items-baseline gap-1.5">
                      <dt className="sr-only">{c.live[k]}</dt>
                      <dd className="text-base text-paper">{live[k]}</dd>
                      <span aria-hidden="true">{c.live[k]}</span>
                    </div>
                  ))}
                  <span className="text-paper/35">live from GitHub</span>
                </motion.dl>
              )}
            </AnimatePresence>
          </div>
        </div>

        {/* Ways to help */}
        <ul className="relative mx-auto mt-20 grid max-w-7xl gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {c.ways.map((w, i) => (
            <motion.li
              key={w.id}
              initial={{ opacity: 0, y: 18 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.4 }}
              transition={{ duration: 0.45, delay: (i % 3) * 0.06, ease: EASE }}
            >
              <a
                href={`${REPO}${w.path}`}
                target="_blank"
                rel="noreferrer"
                className="panel group flex h-full flex-col p-6 text-ink transition-transform duration-300 hover:-translate-y-1 hover:-rotate-[0.4deg]"
              >
                <span className="font-mono text-[11px] text-ink-3">{String(i + 1).padStart(2, "0")}</span>
                <h3 className="mt-3 font-display text-2xl tracking-[-0.02em]">{w.title}</h3>
                <p className="mt-2 flex-1 text-ink-2">{w.body}</p>
                <span className="mt-5 inline-flex items-center gap-2 text-sm font-semibold text-accent">
                  {w.cta}
                  <span aria-hidden="true" className="transition-transform duration-300 group-hover:translate-x-1">
                    →
                  </span>
                </span>
              </a>
            </motion.li>
          ))}
        </ul>

        {/* First PR, as a git graph */}
        <div className="relative mx-auto mt-16 grid max-w-7xl gap-10 lg:grid-cols-[1.3fr_0.7fr]">
          <ol className="relative rounded-2xl border border-paper/15 bg-night/70 p-6 font-mono text-sm md:p-8">
            <span className="absolute bottom-10 left-[2.15rem] top-10 w-px bg-paper/15 md:left-[2.65rem]" aria-hidden="true" />
            {c.steps.map((s, i) => (
              <motion.li
                key={s.label}
                initial={{ opacity: 0, x: -10 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, amount: 0.8 }}
                transition={{ duration: 0.35, delay: i * 0.08, ease: EASE }}
                className="relative flex items-start gap-4 py-2.5"
              >
                <span
                  aria-hidden="true"
                  className={`relative z-10 mt-1 size-3 shrink-0 rounded-full border-2 ${i === c.steps.length - 1 ? "border-accent bg-accent" : "border-paper/60 bg-stage"}`}
                />
                <div className="min-w-0">
                  <p className="text-[11px] uppercase tracking-[0.18em] text-paper/50">{s.label}</p>
                  <code className="mt-1 block overflow-x-auto whitespace-nowrap text-paper">
                    <span className="text-accent">$ </span>
                    {s.code}
                  </code>
                </div>
              </motion.li>
            ))}
          </ol>

          <div>
            <p className="eyebrow !text-paper/55">House rules</p>
            <ul className="mt-5 space-y-4">
              {c.rules.map((r) => (
                <li key={r} className="flex gap-3 text-paper/80">
                  <span aria-hidden="true" className="mt-2 size-1.5 shrink-0 rounded-full bg-accent" />
                  {r}
                </li>
              ))}
            </ul>
          </div>
        </div>
      </section>
    </MotionConfig>
  );
}

function GitMark() {
  return (
    <svg viewBox="0 0 16 16" className="size-4" aria-hidden="true" fill="currentColor">
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
    </svg>
  );
}
