"use client";

import { useEffect, useState } from "react";
import { motion, useInView } from "framer-motion";
import { useRef } from "react";
import { ArrowRight, Github } from "lucide-react";

import { MagneticButton } from "@/components/magnetic-button";
import { repoUrl } from "@/lib/repo";

const SNIPPET = `pub fn note_speech_ended(&self, app: &AppHandle) {
    self.abort_inflight();
    emit_status(app, processing_status());
}`;

export function OpenSource() {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { once: true });
  const [stars, setStars] = useState(0);
  const [contributors, setContributors] = useState(0);
  const [repo, setRepo] = useState(repoUrl());

  useEffect(() => {
    let frame = 0;
    fetch("/api/github", { cache: "no-store" })
      .then((res) => res.json())
      .then((data) => {
        if (data?.repo) setRepo(data.repo);
        if (typeof data?.stats?.contributors === "number") setContributors(data.stats.contributors);
        const target = typeof data?.stats?.stars === "number" ? data.stats.stars : 0;
        if (!inView) {
          setStars(target);
          return;
        }
        const start = performance.now();
        const tick = (now: number) => {
          const t = Math.min(1, (now - start) / 900);
          setStars(Math.round(target * t));
          if (t < 1) frame = requestAnimationFrame(tick);
        };
        frame = requestAnimationFrame(tick);
      })
      .catch(() => undefined);
    return () => cancelAnimationFrame(frame);
  }, [inView]);

  return (
    <section id="open-source" ref={ref} className="px-5 py-24">
      <div className="mx-auto grid max-w-6xl gap-10 md:grid-cols-2">
        <div>
          <h2 className="text-3xl font-semibold md:text-4xl">Transparent. Open. Trusted.</h2>
          <div className="mt-6 flex flex-wrap items-center gap-3 text-sm text-mist">
            <span className="inline-flex items-center gap-2 rounded-full border border-white/10 px-3 py-1">
              <Github className="size-4" /> {stars.toLocaleString()} Stars
            </span>
            <span className="rounded-full border border-white/10 px-3 py-1">
              {contributors} Contributor{contributors === 1 ? "" : "s"}
            </span>
            <span className="rounded-full border border-white/10 px-3 py-1">Apache 2.0</span>
          </div>
          <pre className="mt-8 overflow-x-auto rounded-2xl border border-white/10 bg-black/60 p-5 font-mono text-xs leading-6 text-mist">
            <TypeBlock text={`git clone ${repo}.git\ncd ghostnote\nnpm install\nnpm run tauri dev`} />
          </pre>
        </div>
        <div>
          <div className="glass rounded-3xl p-5">
            <p className="text-[10px] uppercase tracking-[0.18em] text-mist">stealth.rs</p>
            <pre className="mt-4 overflow-x-auto font-mono text-[13px] leading-7 text-cyan/90">
              {SNIPPET}
            </pre>
            <div className="mt-6 flex flex-wrap gap-3">
              <MagneticButton href={repo} variant="secondary">
                <span className="inline-flex items-center gap-2">
                  View on GitHub <ArrowRight className="size-3.5" />
                </span>
              </MagneticButton>
              <MagneticButton href="#cta" variant="ghost">
                Join Our Discord
              </MagneticButton>
            </div>
          </div>
          <motion.div
            className="mt-4 grid grid-cols-12 gap-1"
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
          >
            {Array.from({ length: 84 }).map((_, index) => (
              <span
                key={index}
                className="aspect-square rounded-[2px] bg-cyan"
                style={{ opacity: 0.12 + ((index * 17) % 8) * 0.1 }}
              />
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function TypeBlock({ text }: { text: string }) {
  const [n, setN] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => {
      setN((value) => (value < text.length ? value + 1 : value));
    }, 28);
    return () => window.clearInterval(id);
  }, [text]);
  return (
    <>
      {text.slice(0, n)}
      <span className="animate-pulse">▌</span>
    </>
  );
}
