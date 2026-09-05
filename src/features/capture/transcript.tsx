import { useEffect, useRef } from "react";

import { useCaptureStore } from "@/store/capture";

export function Transcript() {
  const { segments, status, modelInstalled } = useCaptureStore();
  const endRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Follow the live transcript, but only while the user is already at the
  // bottom — yanking the view down while they are re-reading something further
  // up would be worse than not following at all.
  useEffect(() => {
    const container = scrollRef.current;
    if (!container) return;

    const distanceFromBottom =
      container.scrollHeight - container.scrollTop - container.clientHeight;
    if (distanceFromBottom < 120) {
      endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
    }
  }, [segments.length]);

  if (segments.length === 0) {
    return (
      <div className="flex-1 px-6 py-4">
        <p className="max-w-prose text-sm leading-relaxed text-muted-foreground">
          {!modelInstalled
            ? "No speech model found. Install a Whisper model to start transcribing."
            : status.running
              ? "Listening for their questions."
              : "Press Record to start a meeting. Audio is captured and transcribed entirely on this machine."}
        </p>
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="selectable flex-1 overflow-y-auto px-6 py-4">
      <div className="flex flex-col gap-3">
        {segments.map((segment) => (
            <article key={segment.id} className="flex gap-3">
              <span className="w-12 shrink-0 pt-1 text-[10px] font-semibold uppercase tracking-wider text-sky-300/90">
                Them
              </span>
              <p className="min-w-0 flex-1 rounded-2xl rounded-tl-sm bg-white/10 px-3 py-2 text-sm leading-relaxed text-foreground">
                {segment.text}
              </p>
              <time className="shrink-0 self-end pb-1 text-[10px] tabular-nums text-muted-foreground/50">
                {formatOffset(segment.startMs)}
              </time>
            </article>
        ))}
      </div>
      <div ref={endRef} />
    </div>
  );
}

function formatOffset(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
