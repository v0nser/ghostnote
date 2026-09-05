"use client";

import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Check, Play } from "lucide-react";

import { MagneticButton } from "@/components/magnetic-button";
import { AppWindow } from "@/components/mockups/app-window";
import { cn } from "@/lib/utils";

const TABS = ["Interview Mode", "Meeting Mode", "Stealth Demo"] as const;
type Tab = (typeof TABS)[number];

const INTERVIEW = [
  {
    q: "Tell me about a challenging project you've led.",
    a: "I led a team of five to rebuild our legacy billing system. We shipped in twelve weeks and cut invoice errors by forty percent.",
  },
  {
    q: "What's your experience with React?",
    a: "I have spent the last four years on React, mostly hooks and performance work. Recently I owned our design-system migration.",
  },
  {
    q: "How do you handle disagreement on a team?",
    a: "I start from the user outcome, then I ask each person to argue the other side. We decide with a written tradeoff, not a hallway vote.",
  },
];

export function Demo() {
  const [tab, setTab] = useState<Tab>("Interview Mode");

  return (
    <section id="demo" className="relative px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-semibold md:text-4xl">See GhostNote in Action</h2>
        <p className="mx-auto mt-3 max-w-xl text-center text-sm text-mist">
          Play with a live mock of the copilot. Same layout you get in the desktop app.
        </p>
        <div className="mx-auto mt-8 flex w-fit flex-wrap justify-center rounded-full border border-white/10 bg-white/5 p-1">
          {TABS.map((item) => (
            <button
              key={item}
              type="button"
              onClick={() => setTab(item)}
              className={cn(
                "focus-ring rounded-full px-4 py-2 text-sm",
                tab === item ? "bg-white text-ink" : "text-mist hover:text-white",
              )}
            >
              {item}
            </button>
          ))}
        </div>
        <div className="mt-10">
          {tab === "Interview Mode" ? <InterviewDemo /> : null}
          {tab === "Meeting Mode" ? <MeetingDemo /> : null}
          {tab === "Stealth Demo" ? <StealthDemo /> : null}
        </div>
      </div>
    </section>
  );
}

function InterviewDemo() {
  const [running, setRunning] = useState(false);
  const [step, setStep] = useState(0);
  const [listening, setListening] = useState(false);
  const [showAnswer, setShowAnswer] = useState(false);
  const pair = INTERVIEW[step % INTERVIEW.length];

  useEffect(() => {
    if (!running) return;
    setListening(true);
    setShowAnswer(false);
    const listen = window.setTimeout(() => {
      setListening(false);
      setShowAnswer(true);
    }, 2000);
    const next = window.setTimeout(() => {
      setStep((value) => (value + 1) % INTERVIEW.length);
    }, 5000);
    return () => {
      window.clearTimeout(listen);
      window.clearTimeout(next);
    };
  }, [running, step]);

  return (
    <div className="glass mx-auto grid max-w-5xl gap-6 rounded-3xl p-5 md:grid-cols-2">
      <div>
        <p className="text-[10px] uppercase tracking-[0.18em] text-mist">Interviewer</p>
        <div className="mt-4 rounded-2xl bg-white/5 p-4">
          <p className="text-sm leading-relaxed text-white">{running ? pair.q : "Click start to hear the first question."}</p>
        </div>
        {listening ? <Waveform /> : null}
        {!running ? (
          <div className="mt-6">
            <MagneticButton onClick={() => setRunning(true)}>
              <span className="inline-flex items-center gap-2">
                <Play className="size-3.5" /> Start Demo
              </span>
            </MagneticButton>
          </div>
        ) : null}
      </div>
      <div>
        <p className="text-[10px] uppercase tracking-[0.18em] text-mist">GhostNote</p>
        <div className="mt-4 min-h-[160px] rounded-2xl border border-cyan/20 bg-cyan/5 p-4">
          {showAnswer ? (
            <p className="text-base font-medium leading-relaxed text-white">{pair.a}</p>
          ) : (
            <p className="animate-pulse text-sm text-cyan">
              {running ? "Listening…" : "One spoken answer appears here."}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

function MeetingDemo() {
  const lines = [
    { who: "You", text: "I can take the API contract this week." },
    { who: "Them", text: "Can we also lock the auth flow before Friday?" },
    { who: "Them", text: "And who owns the migration checklist?" },
    { who: "You", text: "I'll own auth. Priya can take the checklist." },
  ];

  return (
    <div className="glass mx-auto max-w-5xl rounded-3xl p-5">
      <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-mist">
        <p className="text-white">Product sync · 3 participants</p>
        <p className="font-mono text-cyan">00:12:48</p>
      </div>
      <div className="mt-5 space-y-2">
        {lines.map((line, index) => (
          <motion.p
            key={line.text}
            initial={{ opacity: 0, y: 8 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.15 }}
            className="text-sm"
          >
            <span className={line.who === "You" ? "text-cyan" : "text-mist"}>{line.who}: </span>
            <span className="text-white">{line.text}</span>
          </motion.p>
        ))}
      </div>
      <div className="mt-6 rounded-2xl bg-white/5 p-4">
        <p className="text-[10px] uppercase tracking-[0.18em] text-mist">Live summary</p>
        <ul className="mt-3 space-y-2 text-sm text-white">
          <li className="flex gap-2">
            <Check className="mt-0.5 size-4 text-cyan" /> Lock auth flow before Friday
          </li>
          <li className="flex gap-2">
            <Check className="mt-0.5 size-4 text-cyan" /> You own the API contract
          </li>
          <li className="flex gap-2">
            <Check className="mt-0.5 size-4 text-cyan" /> Priya owns the migration checklist
          </li>
        </ul>
      </div>
    </div>
  );
}

function StealthDemo() {
  const [on, setOn] = useState(false);
  const [app, setApp] = useState<"Zoom" | "Meet" | "Teams">("Zoom");

  return (
    <div>
      <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
        <div className="flex gap-2">
          {(["Zoom", "Meet", "Teams"] as const).map((name) => (
            <button
              key={name}
              type="button"
              onClick={() => setApp(name)}
              className={cn(
                "focus-ring rounded-full px-3 py-1.5 text-xs",
                app === name ? "bg-white text-ink" : "bg-white/5 text-mist",
              )}
            >
              {name}
            </button>
          ))}
        </div>
        <MagneticButton onClick={() => setOn((value) => !value)} variant={on ? "secondary" : "primary"}>
          {on ? "Stealth on" : "Activate Stealth Mode"}
        </MagneticButton>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        <div>
          <p className="mb-2 text-xs text-mist">Your screen</p>
          <AppWindow stealth={on} onToggle={() => setOn((value) => !value)} compact />
        </div>
        <div>
          <p className="mb-2 text-xs text-mist">What they see · {app}</p>
          <div className="relative min-h-[280px] overflow-hidden rounded-2xl border border-white/10 bg-[#1a1c22] p-4">
            <p className="text-sm text-white/80">{app} · Interview room</p>
            <div className="mt-6 grid grid-cols-2 gap-3">
              <div className="aspect-video rounded-xl bg-white/10" />
              <div className="aspect-video rounded-xl bg-white/10" />
            </div>
            <motion.div
              animate={{ opacity: on ? 0 : 1 }}
              className="absolute bottom-4 right-4 w-40 rounded-xl border border-white/10 bg-black/70 p-3 text-[11px] text-white"
            >
              GhostNote is visible here if stealth is off.
            </motion.div>
            {on ? (
              <motion.div
                initial={{ scale: 0.8, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                className="absolute inset-x-0 bottom-6 flex flex-col items-center gap-2 text-center"
              >
                <span className="flex size-10 items-center justify-center rounded-full bg-emerald-400/20 text-emerald-300">
                  <Check className="size-5" />
                </span>
                <p className="text-sm text-white">100% invisible during screen share</p>
              </motion.div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}

function Waveform() {
  return (
    <div className="mt-5 flex h-8 items-end gap-1" aria-label="Listening">
      {Array.from({ length: 14 }).map((_, index) => (
        <span
          key={index}
          className="w-1 rounded-full bg-cyan"
          style={{
            height: "100%",
            animation: `wave 0.9s ease-in-out ${index * 0.06}s infinite`,
          }}
        />
      ))}
    </div>
  );
}
