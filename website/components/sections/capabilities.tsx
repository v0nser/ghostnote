"use client";

import {
  Bot,
  Brain,
  CalendarClock,
  EyeOff,
  FileStack,
  Globe,
  Mic,
  ShieldCheck,
  Terminal,
  Workflow,
} from "lucide-react";
import { motion } from "framer-motion";

import { TiltCard } from "@/components/tilt-card";

const CAPABILITIES = [
  {
    icon: EyeOff,
    title: "Stealth meeting assistant",
    copy: "Stay out of Zoom, Teams, and Meet shares. Whisper + Ollama answer locally when they stop talking.",
  },
  {
    icon: Bot,
    title: "General-purpose agent",
    copy: "Give it a goal. It plans, uses tools, pauses for approval, and keeps an audit trail.",
  },
  {
    icon: Workflow,
    title: "Multi-step planning",
    copy: "Research, compare, draft, then wait. You see every step. You can pause, retry, or cancel.",
  },
  {
    icon: Globe,
    title: "Web research",
    copy: "Search and fetch public pages from an isolated context. Website text cannot grant permissions.",
  },
  {
    icon: FileStack,
    title: "Workspace files & artifacts",
    copy: "Read and write inside a jail. Reports, drafts, and notes stay on disk as artifacts.",
  },
  {
    icon: Terminal,
    title: "Sandboxed code",
    copy: "Short Python or Node snippets in a temp workspace. No raw host shell.",
  },
  {
    icon: Brain,
    title: "Persistent memory",
    copy: "Say “remember this.” Retrieve only what is relevant. Forget is an approval.",
  },
  {
    icon: CalendarClock,
    title: "Scheduled tasks",
    copy: "“Check this every morning.” One-shot, daily, weekly — while the app is open.",
  },
  {
    icon: ShieldCheck,
    title: "Ghost Sentinel",
    copy: "High-risk actions always ask. The model cannot approve itself or escalate permissions.",
  },
  {
    icon: Mic,
    title: "Voice commands",
    copy: "Hold Voice, speak a goal, stop. The same local Whisper path as meetings.",
  },
];

export function Capabilities() {
  return (
    <section id="capabilities" className="px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <p className="text-xs uppercase tracking-[0.28em] text-mist">Shipping now</p>
        <h2 className="mt-3 max-w-2xl text-3xl font-semibold md:text-4xl">
          A local agent platform — meetings are one skill
        </h2>
        <p className="mt-4 max-w-2xl text-sm leading-relaxed text-mist">
          Ghost Note plans, uses tools, remembers, and asks before anything
          sensitive. Audio, memory, tasks, and logs stay on your machine.
        </p>
        <div className="mt-12 grid gap-5 md:grid-cols-2">
          {CAPABILITIES.map((item, index) => (
            <motion.div
              key={item.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-80px" }}
              transition={{ delay: index * 0.04 }}
            >
              <TiltCard>
                <item.icon className="size-6 text-cyan" aria-hidden />
                <h3 className="mt-4 text-xl font-semibold">{item.title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-mist">{item.copy}</p>
              </TiltCard>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
