import { MessageSquareQuote, NotebookPen } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { ActionItem, CoachPhase, MeetingDraft, MeetingNudge, TalkingPoints as Draft } from "@/lib/ipc/coach";
import { useCaptureStore } from "@/store/capture";
import { useCoachStore } from "@/store/coach";

/**
 * Live interview answer: one thing to say, streamed as soon as they stop.
 */
export function TalkingPoints() {
  const {
    available,
    phase,
    suggestion,
    message,
    pendingCue,
    summary,
    summarizing,
    actionItems,
    nudges,
    drafts,
    extractError,
    summarize,
    decideDraft,
  } = useCoachStore();
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
          <div className="space-y-2">
            <p className="selectable max-h-40 overflow-y-auto whitespace-pre-wrap text-xs leading-relaxed text-foreground/90">
              {summary}
            </p>
            <ActionItemsList items={actionItems} error={extractError} />
          </div>
        ) : (
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            {summarizing
              ? "Ollama is writing a recap of the questions asked."
              : "After some questions, summarize the meeting here."}
          </p>
        )}
        <NudgeList nudges={nudges} />
        <DraftList drafts={drafts} onDecide={decideDraft} />
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
        : "Record a meeting. When they ask a question, Coda writes one thing to say."}
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

function ActionItemsList({ items, error }: { items: ActionItem[]; error: string | null }) {
  if (error && !items.length) {
    return <p className="text-[11px] leading-relaxed text-muted-foreground">{error}</p>;
  }
  if (!items.length) return null;
  return (
    <div className="space-y-1.5">
      <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">Action items</p>
      {items.map((item) => (
        <p
          key={`${item.owner ?? "anyone"}-${item.task}`}
          className="rounded-md border border-white/10 px-2 py-1.5 text-[11px] leading-relaxed"
        >
          <span className="font-medium">{item.task}</span>
          {item.owner ? <span className="text-muted-foreground"> · {item.owner}</span> : null}
          {item.due ? <span className="text-muted-foreground"> · {item.due}</span> : null}
        </p>
      ))}
    </div>
  );
}

function NudgeList({ nudges }: { nudges: MeetingNudge[] }) {
  if (!nudges.length) return null;
  return (
    <div className="space-y-1.5">
      <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">From earlier meetings</p>
      {nudges.map((nudge) => (
        <p
          key={`${nudge.kind}-${nudge.title}`}
          className="rounded-md border border-white/10 px-2 py-1.5 text-[11px] leading-relaxed"
        >
          <span className="font-medium">{nudge.title}</span>
          <span className="block text-muted-foreground">{nudge.detail}</span>
        </p>
      ))}
    </div>
  );
}

function DraftList({
  drafts,
  onDecide,
}: {
  drafts: MeetingDraft[];
  onDecide: (draftId: string, approve: boolean) => Promise<void>;
}) {
  const pending = drafts.filter((draft) => draft.status === "pending");
  if (!pending.length) return null;
  return (
    <div className="space-y-1.5">
      <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">Needs confirmation</p>
      {pending.map((draft) => (
        <div key={draft.id} className="space-y-1.5 rounded-md border border-white/10 px-2 py-1.5">
          <p className="text-[11px] leading-relaxed">
            <span className="font-medium">{draft.title}</span>
            <span className="block text-muted-foreground">{draft.body}</span>
          </p>
          <div className="flex gap-1.5">
            <Button type="button" size="sm" variant="secondary" onClick={() => void onDecide(draft.id, true)}>
              Approve
            </Button>
            <Button type="button" size="sm" variant="ghost" onClick={() => void onDecide(draft.id, false)}>
              Deny
            </Button>
          </div>
        </div>
      ))}
    </div>
  );
}

function truncate(value: string, max: number): string {
  if (value.length <= max) return value;
  return `${value.slice(0, max - 1)}…`;
}
