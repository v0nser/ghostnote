import { useEffect } from "react";
import { EyeOff, Maximize2, Mic } from "lucide-react";

import { DragRegion } from "@/components/window-chrome";
import { stealthIpc } from "@/lib/ipc/stealth";
import { useCoachStore } from "@/store/coach";
import { useStealthStore } from "@/store/stealth";

/**
 * The pill shell: what the window becomes while Stealth Mode is on.
 *
 * Compact (268x52) while idle. Grows into a card when an answer arrives
 * so the user can read what to say without leaving stealth — and without
 * that card appearing in the screen share, because capture exclusion is
 * already on.
 */
export function StealthPill() {
  const { pending, setEnabled } = useStealthStore();
  const { phase, suggestion, pendingCue } = useCoachStore();

  const answer = suggestion?.answer ?? "";
  const question = suggestion?.question || suggestion?.cue || pendingCue || "";
  const busy = phase !== "idle";
  const expanded = Boolean(answer) || busy || Boolean(question);

  useEffect(() => {
    void stealthIpc.setPillExpanded(expanded);
  }, [expanded]);

  const status =
    phase === "processing"
      ? "Processing…"
      : phase === "writing"
        ? answer || "Writing…"
        : answer || "Hidden from capture";

  return (
    <DragRegion className="readable flex h-full w-full flex-col overflow-hidden rounded-2xl border border-white/20 bg-black/15 backdrop-blur-[2px]">
      <div className="flex h-[52px] shrink-0 items-center gap-2.5 px-3">
        <span
          className="flex size-7 shrink-0 items-center justify-center rounded-full bg-stealth/15"
          aria-hidden
          data-tauri-drag-region="false"
        >
          <EyeOff className="size-3.5 text-stealth" />
        </span>

        <div className="min-w-0 flex-1 leading-tight" data-tauri-drag-region="false">
          <p className="truncate text-xs font-medium text-foreground">GhostNote</p>
          <p className="truncate text-[10px] text-muted-foreground">{status}</p>
        </div>

        <span
          className="flex size-7 shrink-0 items-center justify-center rounded-full text-muted-foreground"
          aria-hidden
          data-tauri-drag-region="false"
        >
          <Mic className="size-3.5" />
        </span>

        <button
          type="button"
          aria-label="Exit Stealth Mode"
          disabled={pending}
          data-tauri-drag-region="false"
          onClick={() => void setEnabled(false)}
          className="flex size-7 shrink-0 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-white/8 hover:text-foreground disabled:opacity-40"
        >
          <Maximize2 className="size-3.5" />
        </button>
      </div>

      {expanded ? (
        <div
          className="selectable min-h-0 flex-1 overflow-y-auto border-t border-white/8 px-3 py-2"
          data-tauri-drag-region="false"
        >
          {answer ? (
            <div className="flex flex-col gap-1.5">
              {question ? (
                <p className="text-[10px] leading-snug text-muted-foreground/70">{question}</p>
              ) : null}
              <p className="text-sm leading-relaxed text-foreground">{answer}</p>
            </div>
          ) : (
            <p className="animate-pulse text-[11px] leading-relaxed text-muted-foreground">
              {phase === "writing" ? "Writing…" : "Processing…"}
            </p>
          )}
        </div>
      ) : null}
    </DragRegion>
  );
}
