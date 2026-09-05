import { CircleDot, ShieldCheck } from "lucide-react";

import { DragRegion, WindowControls } from "@/components/window-chrome";
import { CaptureMeters, RecordButton } from "@/features/capture/record-button";
import { Transcript } from "@/features/capture/transcript";
import { TalkingPoints } from "@/features/coach/talking-points";
import { StealthBanner, StealthToggle } from "@/features/stealth/stealth-toggle";
import { useCaptureStore } from "@/store/capture";
import { useStealthStore } from "@/store/stealth";

/**
 * The dashboard shell: a minimalist notepad.
 *
 * Capture, live transcript, and talking points live here. Summarisation,
 * meeting memory and sync land in later steps.
 */
export function Dashboard() {
  const stealth = useStealthStore((state) => state.status);
  const capture = useCaptureStore((state) => state.status);

  return (
    <div className="readable flex h-full w-full flex-col overflow-hidden rounded-xl border border-white/20 bg-black/10 backdrop-blur-[2px]">
      <DragRegion className="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-white/10 bg-black/20 px-3">
        <div className="flex min-w-0 items-center gap-2" data-tauri-drag-region="false">
          <WindowControls />
          <span className="ml-1 truncate text-xs font-medium tracking-wide text-muted-foreground">
            GhostNote
          </span>
        </div>

        <div className="flex shrink-0 items-center gap-3" data-tauri-drag-region="false">
          <CaptureMeters />
          <RecordButton />
          <StealthToggle />
        </div>
      </DragRegion>

      <StealthBanner />
      <CaptureBanner />

      <main className="flex min-h-0 flex-1">
        <section className="flex min-w-0 flex-1 flex-col">
          <div className="flex items-center gap-2 px-6 pt-6 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            <CircleDot
              className={capture.running ? "size-3 text-stealth" : "size-3"}
              aria-hidden
            />
            {capture.running ? "Recording" : "No meeting in progress"}
          </div>

          <Transcript />
        </section>

        <aside className="flex w-80 shrink-0 flex-col gap-4 border-l border-white/10 bg-black/25 p-4">
          <TalkingPoints />

          <div className="flex shrink-0 flex-col gap-3 border-t border-border pt-3">
            <h2 className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
              Session
            </h2>

            <dl className="flex flex-col gap-2 text-xs">
              <Row label="Microphone" value={capture.microphone ? "Live" : "Idle"} />
              <Row label="Them" value={capture.systemAudio ? "Live" : "Idle"} />
              {stealth ? (
                <>
                  <Row
                    label="Capture exclusion"
                    value={stealth.captureExcluded ? "Active" : "Off"}
                  />
                  <Row
                    label="Notifications"
                    value={stealth.notificationsSuppressed ? "Silenced" : "Normal"}
                  />
                </>
              ) : null}
            </dl>

            {stealth ? (
              <div className="flex items-start gap-1.5 text-[11px] text-muted-foreground">
                <ShieldCheck className="mt-px size-3 shrink-0" aria-hidden />
                <span className="min-w-0 break-words">{stealth.backend}</span>
              </div>
            ) : null}
          </div>
        </aside>
      </main>
    </div>
  );
}

/**
 * Surfaces capture problems that do not stop the meeting: a missing model, or
 * a session running microphone-only because participant audio was refused.
 */
function CaptureBanner() {
  const { error, status, modelInstalled, systemAudioSupport } = useCaptureStore();

  const message = !modelInstalled
    ? "No speech model installed — recording is disabled until one is available."
    : (status.running && status.systemAudioError) || error;

  if (!message) return null;

  const hint =
    status.systemAudioError && systemAudioSupport?.requiredPermission
      ? ` Grant ${systemAudioSupport.requiredPermission} permission in System Settings to capture the other side.`
      : "";

  return (
    <div className="shrink-0 border-b border-amber-500/25 bg-amber-500/10 px-4 py-2">
      <p className="text-xs leading-relaxed text-amber-200/90">
        {message}
        {hint}
      </p>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="font-medium text-foreground">{value}</dd>
    </div>
  );
}
