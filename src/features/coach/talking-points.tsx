import { MessageSquareQuote, NotebookPen } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { CoachPhase, TalkingPoints as Draft } from "@/lib/ipc/coach";
import { useCaptureStore } from "@/store/capture";
import { useCoachStore } from "@/store/coach";

/**
 * Live interview answer: one thing to say, streamed as soon as they stop.
 */
export function TalkingPoints() {
  const { available, phase, suggestion, message, pendingCue, summary, summarizing, summarize } =
    useCoachStore();
  const running = useCaptureStore((state) => state.status.running);
  const hasQuestions = useCaptureStore((state) => state.segments.length > 0);
  const busy = phase !== "idle";

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-3">
      <h2 className="flex items-center gap-1.5 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
        <MessageSquareQuote className="size-3" aria-hidden />
        Say this
        {busy ? (
          <span className="ml-auto animate-pulse text-[10px] normal-case tracking-normal text-stealth">
            {phase === "processing" ? "Processing" : "Live"}
          </span>
        ) : null}
      </h2>

      <div className="selectable min-h-0 flex-1 overflow-y-auto">
        <TalkingPointsBody
          available={available}
          phase={phase}
          suggestion={suggestion}
          message={summarizing ? null : message}
          pendingCue={pendingCue}
          running={running}
        />
      </div>

      <div className="flex shrink-0 flex-col gap-2 border-t border-white/10 pt-3">
        <div className="flex items-center justify-between gap-2">
          <h2 className="flex items-center gap-1.5 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            <NotebookPen className="size-3" aria-hidden />
            Summary
          </h2>
          <Button
            type="button"
            size="sm"
            variant="secondary"
            disabled={!available || summarizing || (!hasQuestions && !summary)}
            onClick={() => void summarize()}
          >
            {summarizing ? "Summarizing…" : "Summarize"}
          </Button>
        </div>
        {summary ? (
          <p className="selectable max-h-40 overflow-y-auto whitespace-pre-wrap text-xs leading-relaxed text-foreground/90">
            {summary}
          </p>
        ) : (
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            {summarizing
              ? "Ollama is writing a recap of the questions asked."
              : "After some questions, summarize the meeting here."}
          </p>
        )}
      </div>
    </section>
  );
}

function TalkingPointsBody({
  available,
  phase,
  suggestion,
  message,
  pendingCue,
  running,
}: {
  available: boolean;
  phase: CoachPhase;
  suggestion: Draft | null;
  message: string | null;
  pendingCue: string | null;
  running: boolean;
}) {
  if (!available) {
    return (
      <p className="text-xs leading-relaxed text-muted-foreground">
        Start Ollama locally to get a live answer when the interviewer
        finishes a question.
      </p>
    );
  }

  const question = suggestion?.question || suggestion?.cue || pendingCue || "";
  const answer = suggestion?.answer ?? "";
  const processing = phase === "processing" && !answer;
  const writing = phase === "writing";

  if (processing) {
    return (
      <div className="flex flex-col gap-3">
        <ProcessingPulse label="Processing…" />
      </div>
    );
  }

  if (answer || (writing && question)) {
    return (
      <div className="flex flex-col gap-2.5">
        {question ? (
          <p className="text-[10px] leading-snug tracking-wide text-muted-foreground/70">
            {truncate(question, 110)}
          </p>
        ) : null}
        {answer ? (
          <p className="text-base font-medium leading-relaxed text-foreground">
            {answer}
            {writing ? <Caret /> : null}
          </p>
        ) : (
          <ProcessingPulse label="Writing…" />
        )}
      </div>
    );
  }

  if (message) {
    return <p className="text-xs leading-relaxed text-muted-foreground">{message}</p>;
  }

  return (
    <p className="text-xs leading-relaxed text-muted-foreground">
      {running
        ? "When they finish a question, one answer appears here."
        : "Record a meeting. When they ask a question, GhostNote writes one thing to say."}
    </p>
  );
}

function ProcessingPulse({ label }: { label: string }) {
  return (
    <div className="flex items-center gap-2 text-sm text-foreground/80">
      <span className="flex gap-1" aria-hidden>
        <span className="size-1.5 animate-pulse rounded-full bg-stealth" />
        <span className="size-1.5 animate-pulse rounded-full bg-stealth [animation-delay:120ms]" />
        <span className="size-1.5 animate-pulse rounded-full bg-stealth [animation-delay:240ms]" />
      </span>
      <span className="animate-pulse">{label}</span>
    </div>
  );
}

function Caret() {
  return (
    <span
      className="ml-0.5 inline-block h-[0.9em] w-px translate-y-[0.1em] animate-pulse bg-stealth"
      aria-hidden
    />
  );
}

function truncate(value: string, max: number): string {
  if (value.length <= max) return value;
  return `${value.slice(0, max - 1)}…`;
}
