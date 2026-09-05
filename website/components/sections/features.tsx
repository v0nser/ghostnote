"use client";

import { Brain, Cloud, Cpu, EyeOff, Shield, Zap } from "lucide-react";
import { motion } from "framer-motion";

import { TiltCard } from "@/components/tilt-card";

const FEATURES = [
  {
    icon: EyeOff,
    title: "Military-Grade Stealth",
    copy: "Invisible on Zoom, Teams, and Meet. The window never appears in the share.",
    visual: "Window dissolves the moment stealth is on.",
  },
  {
    icon: Shield,
    title: "100% Local AI",
    copy: "Mic to Whisper to Ollama, all on your machine. A hard no on the cloud.",
    visual: "Mic → local chip → answer. Cloud marked out.",
  },
  {
    icon: Zap,
    title: "Real-Time Intelligence",
    copy: "The answer starts streaming the instant they stop talking. One spoken reply.",
    visual: "Question in. Answer out in under a second of feel.",
  },
  {
    icon: Brain,
    title: "Smart Context Awareness",
    copy: "Only the last 45 seconds go to the model. Fast, on-topic, never a dump of the whole call.",
    visual: "Nodes light up around the latest question.",
  },
  {
    icon: Cpu,
    title: "Custom AI Models",
    copy: "Llama 3.1 today. Swap in Mistral or Qwen when you want a different voice.",
    visual: "Model cards with speed and accuracy.",
  },
  {
    icon: Cloud,
    title: "Encrypted Cloud Sync",
    copy: "Optional. Devices stay in lockstep without handing the meeting to a vendor.",
    visual: "Laptop, phone, tablet pulsing in sync.",
  },
];

export function Features() {
  return (
    <section id="features" className="px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="max-w-xl text-3xl font-semibold md:text-4xl">
          Built for Professionals Who Demand More
        </h2>
        <div className="mt-12 grid gap-5 md:grid-cols-2">
          {FEATURES.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-80px" }}
              transition={{ delay: index * 0.06 }}
            >
              <TiltCard>
                <feature.icon className="size-6 text-cyan" aria-hidden />
                <h3 className="mt-4 text-xl font-semibold">{feature.title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-mist">{feature.copy}</p>
                <p className="mt-4 text-xs uppercase tracking-[0.16em] text-cyan/70">
                  {feature.visual}
                </p>
              </TiltCard>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
