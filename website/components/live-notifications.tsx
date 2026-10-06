"use client";

import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";

const NOTES = [
  { name: "John", location: "San Francisco", action: "just installed Ghost Note" },
  { name: "Sarah", location: "London", action: "ran a local research task" },
  { name: "Priya", location: "Bengaluru", action: "turned on stealth for a call" },
  { name: "Diego", location: "Madrid", action: "pulled llama3.2:3b" },
  { name: "Hana", location: "Tokyo", action: "used a voice command" },
];

export function LiveNotifications() {
  const [note, setNote] = useState<(typeof NOTES)[number] | null>(null);

  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let hide: number | undefined;
    const show = () => {
      setNote(NOTES[Math.floor(Math.random() * NOTES.length)]);
      hide = window.setTimeout(() => setNote(null), 5000);
    };
    const start = window.setTimeout(show, 4000);
    const interval = window.setInterval(show, 13000);
    return () => {
      window.clearTimeout(start);
      window.clearTimeout(hide);
      window.clearInterval(interval);
    };
  }, []);

  return (
    <div className="pointer-events-none fixed bottom-5 right-5 z-[60] w-[min(92vw,320px)]">
      <AnimatePresence>
        {note ? (
          <motion.aside
            initial={{ opacity: 0, x: 32 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 24 }}
            className="pointer-events-auto rounded-2xl border border-white/10 bg-charcoal/95 p-4 shadow-glow backdrop-blur"
            role="status"
          >
            <p className="text-sm text-white">
              {note.name} from {note.location} {note.action}
            </p>
          </motion.aside>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
