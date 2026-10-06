import { AnimatePresence, MotionConfig, motion } from "motion/react";
import { useEffect, useRef, useState, type ReactNode } from "react";

import { cast, site } from "../content/site";
import { Character } from "./cast/Character";

type Turn =
  | { id: number; kind: "voice"; prompt: string; phase: 0 | 1 | 2 }
  | { id: number; kind: "screen"; prompt: string; text: string; done: boolean }
  | { id: number; kind: "meeting"; prompt: string }
  | { id: number; kind: "approval"; prompt: string; status: "waiting" | "sent" | "rejected" }
  | { id: number; kind: "forget"; prompt: string; status: "pending" | "deleted" | "kept" }
  | { id: number; kind: "unknown"; prompt: string };

interface AuditLine {
  id: number;
  at: string;
  text: string;
}

const SCREEN_ANSWER =
  "A PDF called “Q3 recap” is open. The action-items table is highlighted, and the Due column for Sam's row is empty.";

const MEMORIES = ["Manager is Jordan Hale (said in standup).", "Jordan prefers written recaps after meetings."];

function classify(phrase: string): Turn["kind"] {
  const q = phrase.toLowerCase();
  if (q.includes("forget")) return "forget";
  if (q.includes("screen")) return "screen";
  if (q.includes("playlist") || /\bplea\b|\bplay\b/.test(q)) return "voice";
  if (q.includes("summar") || q.includes("standup")) return "meeting";
  if (q.includes("email") || q.includes("send") || q.includes("@")) return "approval";
  return "unknown";
}

export function Demo() {
  return (
    <MotionConfig reducedMotion="user">
      <DemoInner />
    </MotionConfig>
  );
}

function DemoInner() {
  const [turns, setTurns] = useState<Turn[]>([]);
  const [audit, setAudit] = useState<AuditLine[]>([]);
  const [input, setInput] = useState("");
  const [sound, setSound] = useState(false);
  const seq = useRef(0);
  const timers = useRef<number[]>([]);
  const list = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const pending = timers.current;
    return () => pending.forEach((t) => window.clearTimeout(t));
  }, []);

  useEffect(() => {
    const el = list.current;
    if (el) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
  }, [turns]);

  const nextId = () => ++seq.current;
  const log = (text: string) =>
    setAudit((rows) =>
      [{ id: nextId(), at: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" }), text }, ...rows].slice(0, 40),
    );
  const later = (ms: number, fn: () => void) => timers.current.push(window.setTimeout(fn, ms));
  const patch = (id: number, fn: (t: Turn) => Turn) => setTurns((ts) => ts.map((t) => (t.id === id ? fn(t) : t)));

  const pop = () => {
    if (!sound) return;
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.frequency.value = 660;
    gain.gain.setValueAtTime(0.05, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.12);
    osc.connect(gain).connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + 0.12);
    osc.onended = () => void ctx.close();
  };

  const run = (phrase: string) => {
    const prompt = phrase.trim();
    if (!prompt) return;
    const kind = classify(prompt);
    const id = nextId();
    log(`You: “${prompt}”`);
    setInput("");

    switch (kind) {
      case "voice":
        setTurns((ts) => [...ts, { id, kind, prompt, phase: 0 }]);
        log("Whisper heard “plea the focus playlist”.");
        later(900, () => {
          patch(id, (t) => (t.kind === "voice" ? { ...t, phase: 1 } : t));
          log("Corrected “plea” → “play”. Fast path — planner skipped.");
        });
        later(1800, () => {
          patch(id, (t) => (t.kind === "voice" ? { ...t, phase: 2 } : t));
          log("Music: Focus playlist playing. Playback verified.");
          pop();
        });
        return;
      case "screen": {
        setTurns((ts) => [...ts, { id, kind, prompt, text: "", done: false }]);
        log("⌘⇧C: focused window captured in memory.");
        let i = 0;
        const step = () => {
          i = Math.min(SCREEN_ANSWER.length, i + 3);
          const done = i >= SCREEN_ANSWER.length;
          patch(id, (t) => (t.kind === "screen" ? { ...t, text: SCREEN_ANSWER.slice(0, i), done } : t));
          if (done) log("moondream answered. Frame discarded.");
          else later(32, step);
        };
        later(500, step);
        return;
      }
      case "meeting":
        setTurns((ts) => [...ts, { id, kind, prompt }]);
        log("llama3.1:8b extracted 3 action items with owners and dates.");
        log("Nudges: 1 duplicate, 1 overdue.");
        return;
      case "approval":
        setTurns((ts) => [...ts, { id, kind, prompt, status: "waiting" }]);
        log("Drafted email to alex@example.com. Waiting for your yes.");
        return;
      case "forget":
        setTurns((ts) => [...ts, { id, kind, prompt, status: "pending" }]);
        log("Found 2 matching memories. Nothing deleted yet.");
        return;
      default:
        setTurns((ts) => [...ts, { id, kind: "unknown", prompt }]);
        log("No rehearsal cue matched.");
    }
  };

  const busy = turns.some((t) => (t.kind === "voice" && t.phase < 2) || (t.kind === "screen" && !t.done));
  const waiting = turns.some((t) => (t.kind === "approval" && t.status === "waiting") || (t.kind === "forget" && t.status === "pending"));
  const status = busy ? "working" : waiting ? "waiting for your yes" : turns.length ? "done" : "idle";

  return (
    <section id="try" className="mx-auto max-w-7xl px-4 py-24 md:px-8 md:py-32">
      <p className="eyebrow">{site.demo.eyebrow}</p>
      <div className="mt-4 flex flex-wrap items-end justify-between gap-6">
        <h2 className="reveal font-display text-[clamp(2.8rem,7vw,6rem)] leading-[0.9] tracking-[-0.02em]">
          {site.demo.title} <span className="caption font-normal text-ink-2">{site.demo.titleItalic}</span>
        </h2>
        <p className="max-w-sm rounded-2xl border border-dashed border-ink/25 px-4 py-3 text-sm text-ink-2">{site.demo.disclaimer}</p>
      </div>

      <div className="mt-10 overflow-hidden rounded-[30px] border border-ink/10 bg-card shadow-[0_40px_80px_-48px_rgba(22,20,15,.55)]">
        <div className="flex items-center justify-between gap-4 border-b border-ink/8 px-4 py-3 md:px-6">
          <div className="flex items-center gap-3">
            <span className="size-11">
              <Character who="coda" label={cast.coda.label} speaking={busy} mood={waiting ? "curious" : "calm"} />
            </span>
            <div>
              <p className="text-sm font-medium">Coda</p>
              <p className="flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.18em] text-ink-3">
                <span className={`size-1.5 rounded-full ${busy ? "bg-accent" : waiting ? "bg-accent/60" : "bg-sage"}`} />
                {status}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <span className="hidden rounded-full bg-ink/5 px-3 py-1 font-mono text-[10px] uppercase tracking-[0.18em] text-ink-3 sm:inline">simulated</span>
            <label className="flex cursor-pointer items-center gap-2 text-xs text-ink-2">
              <input type="checkbox" className="accent-[#d4521a]" checked={sound} onChange={(e) => setSound(e.target.checked)} />
              Sound
            </label>
          </div>
        </div>

        <div className="grid md:grid-cols-[minmax(0,1fr)_300px]">
          <div className="flex min-h-0 flex-col">
            <div ref={list} className="h-[440px] overflow-y-auto px-4 py-6 md:px-6" aria-live="polite">
              {turns.length === 0 ? (
                <div className="grid h-full place-items-center text-center">
                  <p className="max-w-xs text-sm text-ink-3">{site.demo.empty}</p>
                </div>
              ) : (
                <ol className="space-y-6">
                  {turns.map((t) => (
                    <li key={t.id} className="space-y-3">
                      <motion.p
                        initial={{ opacity: 0, y: 8 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="ml-auto w-fit max-w-[85%] rounded-[18px] rounded-br-md bg-ink px-4 py-2.5 text-sm text-paper"
                      >
                        {t.prompt}
                      </motion.p>
                      <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.15 }} className="flex gap-3">
                        <span className="coda-dot mt-1 size-6 shrink-0 rounded-full border border-ink" aria-hidden="true" />
                        <div className="min-w-0 flex-1 rounded-[18px] rounded-tl-md border border-ink/8 bg-paper/60 p-4 text-sm leading-relaxed">
                          <TurnBody
                            turn={t}
                            onApprove={(yes) => {
                              patch(t.id, (x) => (x.kind === "approval" ? { ...x, status: yes ? "sent" : "rejected" } : x));
                              log(yes ? "Approved by you. Sent from your mail app (simulated)." : "Rejected by you. Nothing was sent.");
                              if (yes) pop();
                            }}
                            onForget={(yes) => {
                              patch(t.id, (x) => (x.kind === "forget" ? { ...x, status: yes ? "deleted" : "kept" } : x));
                              log(yes ? "Confirmed. 2 memories and their embeddings deleted." : "Kept. Nothing deleted.");
                            }}
                          />
                        </div>
                      </motion.div>
                    </li>
                  ))}
                </ol>
              )}
            </div>

            <div className="border-t border-ink/8 px-4 py-4 md:px-6">
              <div className="-mx-1 flex gap-2 overflow-x-auto px-1 pb-3">
                {site.demo.presets.map((p) => (
                  <button
                    key={p.id}
                    type="button"
                    onClick={() => run(p.label)}
                    className="shrink-0 rounded-full border border-ink/12 px-3.5 py-1.5 text-xs text-ink-2 transition hover:border-ink/30 hover:bg-ink/5 hover:text-ink"
                  >
                    {p.label}
                  </button>
                ))}
              </div>
              <form
                className="flex gap-2"
                onSubmit={(e) => {
                  e.preventDefault();
                  run(input);
                }}
              >
                <label className="sr-only" htmlFor="demo-cue">
                  Cue Coda
                </label>
                <input
                  id="demo-cue"
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  placeholder="Cue Coda…"
                  className="h-12 min-w-0 flex-1 rounded-full border border-ink/12 bg-paper/60 px-5 text-sm outline-none transition focus:border-ink/40"
                />
                <button type="submit" className="btn btn-ink h-12 px-5">
                  Cue
                </button>
              </form>
            </div>
          </div>

          <aside className="border-t border-ink/8 bg-paper/40 md:border-l md:border-t-0" aria-label="Simulated audit log">
            <div className="flex items-center justify-between px-5 pt-5">
              <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">audit · coda_audit.log</p>
              <span className="font-mono text-[10px] text-ink-3">{audit.length}</span>
            </div>
            <ol className="h-[300px] space-y-0 overflow-y-auto px-5 py-4 md:h-[520px]">
              <AnimatePresence initial={false}>
                {audit.map((a) => (
                  <motion.li
                    key={a.id}
                    layout
                    initial={{ opacity: 0, x: 12 }}
                    animate={{ opacity: 1, x: 0 }}
                    className="relative border-l border-ink/12 pb-4 pl-4 text-[13px] leading-snug"
                  >
                    <span className="absolute -left-[3px] top-1.5 size-1.5 rounded-full bg-ink/40" />
                    <span className="block font-mono text-[10px] text-ink-3">{a.at}</span>
                    {a.text}
                  </motion.li>
                ))}
              </AnimatePresence>
              {audit.length === 0 && <li className="text-[13px] text-ink-3">Every step you trigger lands here, in plain English.</li>}
            </ol>
          </aside>
        </div>
      </div>
    </section>
  );
}

function TurnBody({ turn, onApprove, onForget }: { turn: Turn; onApprove: (yes: boolean) => void; onForget: (yes: boolean) => void }) {
  switch (turn.kind) {
    case "voice":
      return (
        <div className="space-y-2">
          <Row label="heard">
            <span className={turn.phase >= 1 ? "text-ink-3 line-through decoration-accent" : ""}>plea</span> the focus playlist
          </Row>
          {turn.phase >= 1 && (
            <Row label="fixed">
              <span className="font-medium text-ink">play</span> the focus playlist · fast path, no planner
            </Row>
          )}
          {turn.phase === 2 ? (
            <motion.p initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} className="flex items-center gap-3 pt-1 font-medium">
              <span className="eq flex h-5 items-end gap-[3px]" aria-hidden="true">
                {[0, 1, 2, 3].map((i) => (
                  <span key={i} className="h-full w-1 rounded-full bg-accent" style={{ animationDelay: `${i * 0.15}s` }} />
                ))}
              </span>
              Focus playlist is playing.
            </motion.p>
          ) : (
            <Thinking />
          )}
        </div>
      );
    case "screen":
      return (
        <div className="grid gap-4 sm:grid-cols-[120px_minmax(0,1fr)]">
          <div className="h-20 overflow-hidden rounded-xl border border-ink/12 bg-card" role="img" aria-label="Simulated thumbnail of a PDF window">
            <div className="flex gap-1 border-b border-ink/8 px-2 py-1.5">
              {[0, 1, 2].map((i) => (
                <span key={i} className="size-1.5 rounded-full bg-ink/20" />
              ))}
            </div>
            <div className="space-y-1.5 p-2">
              <span className="block h-1.5 w-3/4 rounded-full bg-ink/15" />
              <span className="block h-4 rounded-md bg-accent/25" />
              <span className="block h-1.5 w-1/2 rounded-full bg-ink/10" />
            </div>
          </div>
          <div>
            <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-ink-3">moondream · frame in RAM</p>
            <p className={`mt-1 ${turn.done ? "" : "caret"}`}>{turn.text}</p>
          </div>
        </div>
      );
    case "meeting":
      return (
        <div>
          <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-ink-3">action items · llama3.1:8b</p>
          <table className="mt-2 w-full text-left">
            <thead>
              <tr className="text-[11px] text-ink-3">
                <th className="py-1 pr-3 font-normal">Owner</th>
                <th className="py-1 pr-3 font-normal">Task</th>
                <th className="py-1 font-normal">Due</th>
              </tr>
            </thead>
            <tbody>
              {[
                ["Alex", "Send recap", "Thu"],
                ["Sam", "Draft API notes", "Fri"],
                ["You", "Book follow-up", "Yesterday"],
              ].map(([o, task, due]) => (
                <tr key={task} className="border-t border-ink/8">
                  <td className="py-2 pr-3 font-medium">{o}</td>
                  <td className="py-2 pr-3">{task}</td>
                  <td className="py-2 font-mono text-xs">{due}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="mt-3 flex flex-wrap gap-2 text-xs">
            <span className="rounded-full bg-accent/12 px-3 py-1 text-accent-2">Duplicate: Sam had “API notes” last week</span>
            <span className="rounded-full bg-accent/12 px-3 py-1 text-accent-2">Overdue: book follow-up</span>
          </div>
        </div>
      );
    case "approval":
      return (
        <div>
          <div className="rounded-xl border border-ink/10 bg-card p-3">
            <p className="text-xs text-ink-3">To alex@example.com</p>
            <p className="mt-0.5 font-medium">Recap — action items</p>
            <p className="mt-1 text-ink-2">Alex sends the recap. Sam drafts API notes. I'll book the follow-up.</p>
          </div>
          <AnimatePresence mode="wait">
            {turn.status === "waiting" ? (
              <motion.div key="ask" exit={{ opacity: 0 }} className="mt-3 flex flex-wrap items-center gap-2">
                <span className="stamp mr-1 rounded-md border-2 border-accent px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.15em] text-accent-2">
                  waiting for your yes
                </span>
                <button type="button" className="btn btn-ink h-9 px-4 text-xs" onClick={() => onApprove(true)}>
                  Approve
                </button>
                <button type="button" className="btn btn-ghost h-9 px-4 text-xs" onClick={() => onApprove(false)}>
                  Reject
                </button>
              </motion.div>
            ) : (
              <motion.p key="done" initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} className="mt-3 font-medium">
                {turn.status === "sent" ? "Sent — because you said yes." : "Not sent. The draft stays on your machine."}
              </motion.p>
            )}
          </AnimatePresence>
        </div>
      );
    case "forget":
      return (
        <div>
          <AnimatePresence mode="wait">
            {turn.status === "pending" ? (
              <motion.div key="ask" exit={{ opacity: 0, height: 0 }}>
                <p className="text-ink-2">I found these. Nothing is deleted yet:</p>
                <ul className="mt-2 space-y-1.5">
                  {MEMORIES.map((m) => (
                    <li key={m} className="rounded-lg border border-ink/8 bg-card px-3 py-2">
                      {m}
                    </li>
                  ))}
                </ul>
                <div className="mt-3 flex gap-2">
                  <button type="button" className="btn btn-ink h-9 px-4 text-xs" onClick={() => onForget(true)}>
                    Confirm forget
                  </button>
                  <button type="button" className="btn btn-ghost h-9 px-4 text-xs" onClick={() => onForget(false)}>
                    Keep them
                  </button>
                </div>
              </motion.div>
            ) : (
              <motion.p key="done" initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} className="font-medium">
                {turn.status === "deleted" ? "Forgotten. Two memories and their embeddings are gone." : "Kept. Nothing was deleted."}
              </motion.p>
            )}
          </AnimatePresence>
        </div>
      );
    case "unknown":
      return <p className="text-ink-2">This rehearsal only knows the five cues below. In the app, I'd plan this with llama3.1:8b and ask before anything risky.</p>;
  }
}

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <p className="flex gap-3">
      <span className="w-10 shrink-0 pt-0.5 font-mono text-[10px] uppercase tracking-[0.16em] text-ink-3">{label}</span>
      <span>{children}</span>
    </p>
  );
}

function Thinking() {
  return (
    <span className="thinking flex gap-1 pt-1" aria-label="working">
      <span />
      <span />
      <span />
    </span>
  );
}
