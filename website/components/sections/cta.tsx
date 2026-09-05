"use client";

import { Lock, Shield, Sparkles } from "lucide-react";

import { DownloadButtons } from "@/components/download-buttons";

export function Cta() {
  return (
    <section id="cta" className="relative overflow-hidden px-5 py-28">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(200,200,204,0.08),transparent_45%)]" />
      <div className="relative mx-auto max-w-3xl text-center">
        <h2 className="text-4xl font-semibold leading-tight text-white md:text-5xl">
          Ready to Make Every Meeting Count?
        </h2>
        <p className="mx-auto mt-5 max-w-xl text-mist">
          Install the desktop app. No account. No cloud key. Mac gets a .dmg, Windows gets a .exe.
        </p>
        <div className="mt-10">
          <DownloadButtons />
        </div>
        <p className="mt-6 text-sm text-mist">macOS 13+ · Windows 10/11</p>
        <div className="mt-4 flex flex-wrap items-center justify-center gap-5 text-xs text-mist">
          <span className="inline-flex items-center gap-1.5">
            <Sparkles className="size-3.5 text-cyan" /> Open Source
          </span>
          <span className="inline-flex items-center gap-1.5">
            <Lock className="size-3.5 text-cyan" /> Encrypted
          </span>
          <span className="inline-flex items-center gap-1.5">
            <Shield className="size-3.5 text-cyan" /> Local-First
          </span>
        </div>
      </div>
    </section>
  );
}
