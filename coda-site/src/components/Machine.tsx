import { AnimatePresence, MotionConfig, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { site, type Route, type StepKind } from "../content/site";

const ROUTES: Route[] = site.machine.routes;
const pad = (n: number) => String(n).padStart(2, "0");

const KIND_LABEL: Record<StepKind, string> = {
  input: "input",
  model: "local model",
  local: "on-device",
  tool: "tool",
  gate: "you",
  log: "receipt",
};

type OrganId = (typeof site.machine.organs)[number]["id"];

function organFor(kind: StepKind, label: string): OrganId {
  const t = label.toLowerCase();
  if (kind === "gate") return "you";
  if (kind === "tool" || kind === "log") return "hand";
  if (t.includes("moon") || t.includes("frame") || t.includes("hotkey") || t.includes("discard")) return "eye";
  if (t.includes("whisper") || t.includes("micro") || t.includes("correct")) return "ear";
  if (kind === "local") return "router";
  if (kind === "input") return "ear";
  return "cortex";
}

export function Machine() {
  return (
    <MotionConfig reducedMotion="user">
      <MachineInner />
    </MotionConfig>
  );
}

function MachineInner() {
  const ref = useRef<HTMLElement>(null);
  const [routeId, setRouteId] = useState(ROUTES[0].id);
  const [step, setStep] = useState(0);
  const [playing, setPlaying] = useState(true);
  const [inView, setInView] = useState(false);
  const route = ROUTES.find((r) => r.id === routeId) ?? ROUTES[0];
  const len = route.steps.length;
  const current = route.steps[Math.min(step, len - 1)];
  const live = organFor(current.kind, current.label);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(([entry]) => setInView(entry.isIntersecting), { threshold: 0.22 });
    io.observe(el);
    return () => io.disconnect();
  }, []);

  useEffect(() => {
    if (!playing || !inView) return;
    const hold = current.kind === "gate" ? 3200 : 1800;
    const timer = window.setTimeout(() => setStep((s) => (s + 1) % len), hold);
    return () => window.clearTimeout(timer);
  }, [playing, inView, step, len, current.kind]);

  const choose = (id: string) => {
    setRouteId(id);
    setStep(0);
    setPlaying(true);
  };
  const jump = (i: number) => {
    setStep(((i % len) + len) % len);
    setPlaying(false);
  };

  return (
    <section ref={ref} id="machine" className="relative scroll-mt-24 overflow-hidden bg-night text-paper">
      <div className="night-grid absolute inset-0" aria-hidden="true" />
      <div className="relative mx-auto max-w-7xl px-4 py-24 md:px-8 md:py-32">
        <p className="eyebrow text-paper/45">{site.machine.eyebrow}</p>
        <h2 className="reveal mt-4 font-display text-[clamp(2.6rem,7vw,6.2rem)] leading-[0.88] tracking-[-0.04em]">
          {site.machine.title} <span className="caption font-normal text-accent">{site.machine.titleItalic}</span>
        </h2>
        <p className="mt-5 max-w-2xl text-paper/60">{site.machine.intro}</p>

        <div className="mt-10 grid gap-10 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,1fr)] lg:items-start">
          <Nerves live={live} />

          <div>
            <div className="flex flex-wrap gap-2" role="group" aria-label="Choose a cue">
              {ROUTES.map((r) => (
                <button
                  key={r.id}
                  type="button"
                  onClick={() => choose(r.id)}
                  aria-pressed={r.id === routeId}
                  className="relative rounded-full border border-paper/15 px-4 py-2 font-mono text-xs text-paper/70 transition-colors hover:text-paper"
                >
                  {r.id === routeId && (
                    <motion.span layoutId="route-pill" className="absolute inset-0 rounded-full bg-paper" transition={{ type: "spring", stiffness: 380, damping: 32 }} />
                  )}
                  <span className={`relative ${r.id === routeId ? "text-night" : ""}`}>{r.cue}</span>
                </button>
              ))}
            </div>
            <AnimatePresence mode="wait">
              <motion.p key={route.id} className="mt-4 text-sm text-paper/50" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
                {route.summary}
              </motion.p>
            </AnimatePresence>

            <div className="relative mt-8 rounded-[28px] border border-dashed border-paper/20 p-4 md:p-6">
              <span className="absolute -top-2.5 left-5 bg-night px-2 font-mono text-[10px] uppercase tracking-[0.25em] text-paper/50">
                {site.machine.chassis}
              </span>
              <span className="absolute -top-2.5 right-5 bg-night px-2 font-mono text-[10px] uppercase tracking-[0.2em] text-paper/35">
                <span className="line-through decoration-accent">{site.machine.cloud}</span> · unplugged
              </span>

              <ol key={route.id} className="grid gap-2">
                {route.steps.map((s, i) => {
                  const state = i < step ? "past" : i === step ? "now" : "next";
                  const gate = s.kind === "gate";
                  return (
                    <motion.li
                      key={s.label + i}
                      initial={{ opacity: 0, x: 10 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ delay: i * 0.04, duration: 0.35 }}
                    >
                      <button
                        type="button"
                        onClick={() => jump(i)}
                        aria-current={state === "now" ? "step" : undefined}
                        className={`flex w-full items-center gap-3 rounded-2xl border px-3.5 py-2.5 text-left transition-colors duration-300 ${
                          state === "now"
                            ? gate
                              ? "border-accent/60 bg-accent/10"
                              : "border-paper/25 bg-paper/[0.06]"
                            : state === "past"
                              ? "border-paper/10"
                              : "border-paper/6 opacity-40"
                        }`}
                      >
                        <span className="font-mono text-[10px] text-paper/35">{pad(i + 1)}</span>
                        <span className={`min-w-0 flex-1 truncate text-sm ${gate ? "text-accent" : ""}`}>{s.label}</span>
                        <span className="hidden font-mono text-[10px] uppercase tracking-[0.16em] text-paper/35 sm:inline">{KIND_LABEL[s.kind]}</span>
                      </button>
                    </motion.li>
                  );
                })}
              </ol>

              <div className="mt-6 grid gap-6 md:grid-cols-[1fr_auto] md:items-end">
                <AnimatePresence mode="wait">
                  <motion.div
                    key={route.id + step}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -6 }}
                    transition={{ duration: 0.3 }}
                    aria-live="polite"
                  >
                    <p className="font-mono text-[11px] uppercase tracking-[0.2em] text-paper/40">
                      {pad(step + 1)} / {pad(len)} · {KIND_LABEL[current.kind]}
                    </p>
                    <h3 className="mt-2 font-display text-3xl leading-none tracking-[-0.03em] md:text-5xl">
                      {current.label}
                      {current.kind === "gate" && (
                        <span className="ml-3 inline-flex items-center gap-2 rounded-full border border-accent/60 px-3 py-1 align-middle font-sans text-xs font-medium tracking-normal text-accent">
                          <span className="live-dot" />
                          waiting
                        </span>
                      )}
                    </h3>
                    <p className="mt-2 font-mono text-xs text-accent/90">{current.where}</p>
                    <p className="mt-3 max-w-xl text-paper/70">{current.detail}</p>
                  </motion.div>
                </AnimatePresence>
                <div className="flex items-center gap-2">
                  <button type="button" className="ctrl" onClick={() => jump(step - 1)} aria-label="Previous step">
                    ←
                  </button>
                  <button type="button" className="ctrl w-auto px-4 font-mono text-[11px] uppercase tracking-[0.16em]" onClick={() => setPlaying((p) => !p)}>
                    {playing ? "pause" : "play"}
                  </button>
                  <button type="button" className="ctrl" onClick={() => jump(step + 1)} aria-label="Next step">
                    →
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

const NODES: { id: OrganId; x: number; y: number }[] = [
  { id: "ear", x: 70, y: 88 },
  { id: "eye", x: 250, y: 70 },
  { id: "router", x: 160, y: 150 },
  { id: "cortex", x: 160, y: 230 },
  { id: "hand", x: 70, y: 310 },
  { id: "you", x: 250, y: 310 },
];

function Nerves({ live }: { live: OrganId }) {
  return (
    <div>
      <svg viewBox="0 0 320 380" className="w-full overflow-visible" role="img" aria-label="Nerves of a laptop: ear, eye, reflex, cortex, hand, you">
        <rect x="18" y="18" width="284" height="344" rx="28" fill="none" stroke="currentColor" strokeOpacity="0.18" strokeWidth="1.5" strokeDasharray="6 7" />
        <path
          d="M70 88 L160 150 L250 70 M160 150 L160 230 L70 310 M160 230 L250 310 M70 310 L250 310"
          fill="none"
          stroke="currentColor"
          strokeOpacity="0.16"
          strokeWidth="2"
        />
        {NODES.map((n) => {
          const on = n.id === live;
          const organ = site.machine.organs.find((o) => o.id === n.id)!;
          return (
            <g key={n.id}>
              <circle cx={n.x} cy={n.y} r={on ? 22 : 16} fill={on ? "#d4521a" : "none"} stroke={on ? "#d4521a" : "currentColor"} strokeOpacity={on ? 1 : 0.35} strokeWidth="2" className="transition-all duration-500" />
              {on && <circle cx={n.x} cy={n.y} r="28" fill="none" stroke="#d4521a" strokeOpacity="0.35" />}
              <text x={n.x} y={n.y + 42} textAnchor="middle" fill="currentColor" fontSize="11" fontFamily="IBM Plex Mono, ui-monospace, monospace" opacity="0.7">
                {organ.name}
              </text>
            </g>
          );
        })}
      </svg>
      <ul className="mt-2 grid grid-cols-2 gap-3 md:grid-cols-3">
        {site.machine.organs.map((o) => (
          <li
            key={o.id}
            className={`rounded-2xl border px-3 py-2.5 transition-colors duration-500 ${
              o.id === live ? "border-accent/50 bg-accent/10" : "border-paper/10"
            }`}
          >
            <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-paper/40">{o.model}</p>
            <p className="mt-0.5 text-sm font-medium">{o.name}</p>
            <p className="mt-0.5 text-xs text-paper/50">{o.note}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}
