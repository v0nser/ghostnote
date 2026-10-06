"use client";

import { Brain, Cpu, EyeOff, Mic, Shield, Zap } from "lucide-react";
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
    title: "100% Local by default",
    copy: "Mic to Whisper to Ollama. Tasks, memory, and audit logs stay on disk. Cloud models are opt-in.",
    visual: "Mic → local chip → answer. Cloud marked out.",
  },
  {
    icon: Zap,
    title: "Faster spoken replies",
    copy: "Prefers a small local model (3B-class) so the first token lands sooner. Still streams as they finish.",
    visual: "Question in. Answer starts immediately.",
  },
  {
    icon: Brain,
    title: "Agent + meeting memory",
    copy: "Last 45 seconds for live answers. Longer memory for goals you ask it to remember.",
    visual: "Nodes light up around the latest question.",
  },
  {
    icon: Cpu,
    title: "Your models",
    copy: "Ollama first. llama3.2:3b and qwen2.5:3b are the fast defaults. Larger models work when you pull them.",
    visual: "Model cards with speed and accuracy.",
  },
  {
    icon: Mic,
    title: "Voice commands",
    copy: "Speak a goal in the Agent tab. Same Whisper sidecar as meetings. No cloud speech API.",
    visual: "Hold Voice. Speak. Plan appears.",
  },
];

export function Features() {
  return (
    <section id="features" className="px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="max-w-xl text-3xl font-semibold md:text-4xl">
          Built for people who want an agent they can see
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
