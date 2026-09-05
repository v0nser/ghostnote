import { Mic, Square } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useCaptureStore } from "@/store/capture";
import { cn } from "@/lib/utils";

export function RecordButton() {
  const { status, pending, modelInstalled, start, stop } = useCaptureStore();

  // Without a local model there is nothing to transcribe with, and starting
  // would record a whole meeting that could never be read back.
  const blocked = !modelInstalled;

  return (
    <Button
      size="sm"
      variant={status.running ? "destructive" : "default"}
      disabled={pending || blocked}
      onClick={() => void (status.running ? stop() : start())}
    >
      {status.running ? <Square className="size-3.5" /> : <Mic className="size-3.5" />}
      {status.running ? "Stop" : "Record"}
    </Button>
  );
}

/** Elapsed time plus a peak meter per source. */
export function CaptureMeters() {
  const { status, levels } = useCaptureStore();

  if (!status.running) return null;

  return (
    <div className="flex items-center gap-3">
      <Meter label="You" peak={levels.you} active />
      <Meter label="Them" peak={levels.participant} active={status.systemAudio} />
    </div>
  );
}

function Meter({ label, peak, active }: { label: string; peak: number; active: boolean }) {
  // Amplitude is linear but hearing is not, so a raw peak barely moves the bar
  // at conversational volume. A cube root approximates perceived loudness
  // closely enough for a meter.
  const filled = active ? Math.min(1, Math.cbrt(Math.max(peak, 0))) : 0;

  return (
    <div className="flex items-center gap-1.5">
      <span className="text-[10px] uppercase tracking-wider text-muted-foreground">{label}</span>
      <div className="h-1 w-12 overflow-hidden rounded-full bg-white/10">
        <div
          className={cn("h-full rounded-full transition-[width] duration-100", {
            "bg-stealth": active,
            "bg-muted-foreground/40": !active,
          })}
          style={{ width: `${filled * 100}%` }}
        />
      </div>
    </div>
  );
}
