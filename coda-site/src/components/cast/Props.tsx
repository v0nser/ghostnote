import { motion } from "motion/react";

import type { PropId } from "../../content/site";

export type PropMode = "chaos" | "settling" | "resolved" | "dock";

interface Slot {
  left?: string;
  right?: string;
  top: string;
  rotate: number;
}

export const SLOTS: Record<PropId, Slot> = {
  tabs: { left: "2%", top: "6%", rotate: -6 },
  rec: { left: "30%", top: "0%", rotate: 2 },
  sticky: { right: "3%", top: "4%", rotate: 7 },
  cal: { left: "8%", top: "50%", rotate: 3 },
  pdf: { right: "6%", top: "52%", rotate: -5 },
};

export const PROP_ORDER: PropId[] = ["rec", "tabs", "sticky", "cal", "pdf"];

const card =
  "rounded-2xl border border-ink/15 bg-card p-3 text-ink shadow-[0_14px_34px_-18px_rgba(22,20,15,.4)] md:p-4";

export function PropCard({ id, mode, index = 0 }: { id: PropId; mode: PropMode; index?: number }) {
  const positioned = mode === "chaos" || mode === "settling";
  const slot = SLOTS[id];
  return (
    <motion.div
      layoutId={`prop-${id}`}
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1, rotate: positioned ? slot.rotate : 0 }}
      exit={{ opacity: 0, scale: 0.92 }}
      transition={{ type: "spring", stiffness: 210, damping: 26, delay: mode === "dock" ? index * 0.05 : 0 }}
      className={positioned ? "absolute w-[46%] max-w-[230px]" : ""}
      style={positioned ? { left: slot.left, right: slot.right, top: slot.top } : undefined}
    >
      <div className={mode === "chaos" ? "jitter" : ""} style={{ animationDelay: `${index * -0.6}s` }}>
        {mode === "dock" ? <DockChip id={id} /> : mode === "resolved" ? <Resolved id={id} /> : <Mess id={id} />}
      </div>
    </motion.div>
  );
}

function Mess({ id }: { id: PropId }) {
  switch (id) {
    case "tabs":
      return (
        <div className="relative pl-2 pt-2">
          <div className="absolute inset-0 -translate-x-1 -translate-y-1 rounded-2xl border border-ink/10 bg-paper-2" />
          <div className={`${card} relative`}>
            <div className="flex gap-1">
              {Array.from({ length: 7 }).map((_, i) => (
                <span key={i} className="h-2 flex-1 rounded-full bg-ink/10" />
              ))}
            </div>
            <p className="mt-2 text-[13px] font-medium md:text-sm">47 tabs open</p>
            <p className="text-[11px] text-ink-3 md:text-xs">one of them has the PDF</p>
          </div>
        </div>
      );
    case "sticky":
      return (
        <div className="rounded-md bg-[#ffd98f] p-3 text-ink shadow-[0_14px_30px_-16px_rgba(22,20,15,.5)] md:p-4">
          <p className="font-display text-2xl italic leading-none md:text-3xl">URGENT!!</p>
          <p className="mt-2 text-[11px] text-ink/70 md:text-xs">reply to Alex?? recap??</p>
        </div>
      );
    case "rec":
      return (
        <div className={card}>
          <p className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.2em] text-accent-2">
            <span className="rec-dot size-2 rounded-full bg-accent" /> rec
          </p>
          <p className="mt-1.5 truncate text-[13px] font-medium md:text-sm">standup_final_v3.m4a</p>
          <div className="mt-2 flex h-5 items-end gap-[3px]" aria-hidden="true">
            {[5, 9, 4, 12, 7, 14, 6, 10, 3, 8, 12, 5, 9, 4].map((h, i) => (
              <span key={i} className="w-1 rounded-full bg-ink/25" style={{ height: h + 4 }} />
            ))}
          </div>
          <p className="mt-1.5 text-[11px] text-ink-3 md:text-xs">47:12 · nobody watched it</p>
        </div>
      );
    case "cal":
      return (
        <div className={card}>
          <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">today</p>
          <div className="relative mt-2 h-14">
            <div className="absolute inset-x-0 top-0 rounded-lg bg-sage/30 px-2 py-1 text-[11px] md:text-xs">2:00 Sync</div>
            <div className="absolute inset-x-3 top-5 rounded-lg border border-accent/50 bg-accent/15 px-2 py-1 text-[11px] md:text-xs">2:00 Review</div>
          </div>
          <p className="text-[11px] font-medium text-accent-2 md:text-xs">clash</p>
        </div>
      );
    case "pdf":
      return (
        <div className={`${card} flex items-center gap-3`}>
          <span className="grid h-10 w-8 shrink-0 place-items-center rounded-md border border-ink/15 bg-paper font-mono text-[8px]">PDF</span>
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium md:text-sm">recap_FINAL(2).pdf</p>
            <p className="text-[11px] text-ink-3 md:text-xs">which tab?</p>
          </div>
        </div>
      );
  }
}

function Resolved({ id }: { id: PropId }) {
  switch (id) {
    case "rec":
      return (
        <div className={card}>
          <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-sage">action items · 3</p>
          <table className="mt-2 w-full text-left text-[11px] md:text-[13px]">
            <tbody>
              {[
                ["Alex", "Send recap", "Thu"],
                ["Sam", "API notes", "Fri"],
                ["You", "Book follow-up", "Mon"],
              ].map(([owner, task, due]) => (
                <tr key={task} className="border-t border-ink/8">
                  <td className="py-1 pr-2 font-medium">{owner}</td>
                  <td className="py-1 pr-2 text-ink-2">{task}</td>
                  <td className="py-1 text-right font-mono text-[10px] text-ink-3">{due}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    case "tabs":
      return (
        <div className={`${card} flex items-center gap-3`}>
          <span className="eq flex h-6 items-end gap-[3px]" aria-hidden="true">
            {[0, 1, 2, 3].map((i) => (
              <span key={i} className="h-full w-1 rounded-full bg-accent" style={{ animationDelay: `${i * 0.15}s` }} />
            ))}
          </span>
          <div className="min-w-0">
            <p className="text-[13px] font-medium md:text-sm">Focus playlist</p>
            <p className="text-[11px] text-ink-3 md:text-xs">playing · heard “plea”</p>
          </div>
        </div>
      );
    case "sticky":
      return (
        <div className={`${card} relative`}>
          <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">draft · mail</p>
          <p className="mt-1 text-[13px] font-medium md:text-sm">Recap — action items</p>
          <p className="text-[11px] text-ink-3 md:text-xs">to alex@example.com</p>
          <span className="stamp mt-2 inline-block rounded-md border-2 border-accent px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.15em] text-accent-2">
            waiting for your yes
          </span>
        </div>
      );
    case "cal":
      return (
        <div className={card}>
          <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-ink-3">draft · calendar</p>
          <p className="mt-1 text-[13px] font-medium md:text-sm">Follow-up · Mon 10:00</p>
          <p className="text-[11px] text-ink-3 md:text-xs">needs your yes</p>
        </div>
      );
    case "pdf":
      return (
        <div className={card}>
          <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-sage">recap.pdf · summary</p>
          <div className="mt-2 space-y-1.5" aria-hidden="true">
            {[92, 78, 85, 60].map((w) => (
              <span key={w} className="block h-1.5 rounded-full bg-ink/10" style={{ width: `${w}%` }} />
            ))}
          </div>
        </div>
      );
  }
}

const DOCK: Record<PropId, { label: string; tone: string }> = {
  rec: { label: "3 action items", tone: "bg-sage" },
  tabs: { label: "Focus playlist", tone: "bg-accent" },
  sticky: { label: "Recap · waiting for you", tone: "bg-accent" },
  cal: { label: "Follow-up · draft", tone: "bg-ink-3" },
  pdf: { label: "PDF summary", tone: "bg-sage" },
};

function DockChip({ id }: { id: PropId }) {
  const item = DOCK[id];
  return (
    <span className="flex items-center gap-2 rounded-full border border-ink/15 bg-card px-3.5 py-2 text-[12px] text-ink shadow-[0_10px_24px_-16px_rgba(22,20,15,.5)] md:text-sm">
      <span className={`size-1.5 rounded-full ${item.tone}`} />
      {item.label}
    </span>
  );
}
