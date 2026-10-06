import { useEffect, useMemo } from "react";
import {
  Bot,
  CalendarClock,
  Check,
  Circle,
  LoaderCircle,
  MemoryStick,
  Mic,
  Pause,
  Send,
  ShieldAlert,
  Square,
  X,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import type { AgentTask, Approval } from "@/lib/ipc/agent";
import { useAgentStore } from "@/store/agent";

export function AgentWorkspace() {
  const {
    pane,
    setPane,
    profiles,
    conversations,
    tasks,
    approvals,
    memories,
    schedules,
    tools,
    activity,
    draft,
    setDraft,
    send,
    listening,
    voicePhase,
    voiceLevel,
    voiceHint,
    voicePartial,
    heardText,
    pressVoice,
    releaseVoice,
    activeAgentId,
    selectAgent,
    activeConversationId,
    selectConversation,
    selectedTaskId,
    selectTask,
    forget,
    error,
  } = useAgentStore();

  useEffect(() => {
    const onDown = (event: KeyboardEvent) => {
      if (event.code !== "Space" || !event.ctrlKey || event.metaKey || event.repeat) return;
      event.preventDefault();
      void pressVoice();
    };
    const onUp = (event: KeyboardEvent) => {
      if (event.code !== "Space" || !event.ctrlKey) return;
      event.preventDefault();
      void releaseVoice();
    };
    window.addEventListener("keydown", onDown);
    window.addEventListener("keyup", onUp);
    return () => {
      window.removeEventListener("keydown", onDown);
      window.removeEventListener("keyup", onUp);
    };
  }, [pressVoice, releaseVoice]);

  const conversation = conversations.find((item) => item.id === activeConversationId) ?? conversations[0];
  const selected = tasks.find((task) => task.taskId === selectedTaskId) ?? tasks[0];
  const pendingApprovals = approvals.filter((item) => item.status === "PENDING");

  return (
    <div className="flex min-h-0 flex-1">
      <aside className="flex w-56 shrink-0 flex-col gap-3 border-r border-white/10 bg-black/20 p-3">
        <Nav label="Assistants" active={pane === "assistants"} onClick={() => setPane("assistants")} />
        <Nav label="Conversations" active={pane === "conversations"} onClick={() => setPane("conversations")} />
        <Nav label="Tasks" active={pane === "tasks"} onClick={() => setPane("tasks")} />
        <Nav label="Scheduled" active={pane === "scheduled"} onClick={() => setPane("scheduled")} />
        <Nav label="Memory" active={pane === "memory"} onClick={() => setPane("memory")} />

        <div className="mt-2 min-h-0 flex-1 overflow-y-auto">
          {pane === "assistants"
            ? profiles.map((profile) => (
                <button
                  key={profile.id}
                  type="button"
                  onClick={() => selectAgent(profile.id)}
                  className={rowClass(profile.id === activeAgentId)}
                >
                  {profile.name}
                </button>
              ))
            : null}
          {pane === "conversations"
            ? conversations.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => selectConversation(item.id)}
                  className={rowClass(item.id === conversation?.id)}
                >
                  {item.title}
                </button>
              ))
            : null}
          {pane === "tasks"
            ? tasks.map((task) => (
                <button
                  key={task.taskId}
                  type="button"
                  onClick={() => selectTask(task.taskId)}
                  className={rowClass(task.taskId === selected?.taskId)}
                >
                  <span className="block truncate">{task.goal}</span>
                  <span className="text-[10px] uppercase tracking-wide text-muted-foreground">
                    {(task.status ?? "").replaceAll("_", " ")}
                  </span>
                </button>
              ))
            : null}
          {pane === "scheduled"
            ? schedules.map((item) => (
                <p key={item.id} className="rounded-md px-2 py-1.5 text-xs">
                  {item.taskTemplateGoal}
                  <span className="mt-0.5 block text-[10px] uppercase text-muted-foreground">
                    {item.cadence}
                  </span>
                </p>
              ))
            : null}
          {pane === "memory"
            ? memories.map((item) => (
                <div key={item.id} className="flex items-start justify-between gap-2 rounded-md px-2 py-1.5">
                  <p className="text-xs leading-relaxed">{item.content}</p>
                  <button
                    type="button"
                    className="text-[10px] text-muted-foreground hover:text-foreground"
                    onClick={() => {
                      if (window.confirm(`Delete this memory?\n\n${item.content}`)) {
                        void forget(item.id);
                      }
                    }}
                  >
                    Forget
                  </button>
                </div>
              ))
            : null}
        </div>
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <CommandCenter tasks={tasks} approvals={pendingApprovals} schedules={schedules.length} memories={memories.length} />

        {pendingApprovals[0] ? <ApprovalBanner approval={pendingApprovals[0]} /> : null}

        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
          {conversation?.messages?.length ? (
            <ol className="space-y-3">
              {conversation.messages.map((message) => (
                <li key={message.id} className="text-sm leading-relaxed">
                  <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
                    {message.role}
                  </p>
                  <p className="selectable mt-1 whitespace-pre-wrap">{message.content}</p>
                </li>
              ))}
            </ol>
          ) : (
            <p className="text-sm text-muted-foreground">
              Try: “set an alarm for 7am”, “play Kumar Sanu”, “open Safari”, “search Google for today’s weather”.
              Email and calendar wait for your approval.
            </p>
          )}

          {selected ? <TaskCard task={selected} /> : null}
          {heardText || listening || voicePhase === "transcribing" || voicePhase === "error" ? (
            <div className="mt-4 rounded-lg border border-white/10 bg-black/25 px-3 py-2 text-sm">
              <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
                {voicePhase === "recording"
                  ? "Listening"
                  : voicePhase === "transcribing"
                    ? "Processing"
                    : voicePhase === "error"
                      ? "Voice"
                      : "You said"}
              </p>
              <p className="mt-1">
                {voicePhase === "recording"
                  ? voicePartial || "Listening…"
                  : voicePhase === "transcribing"
                    ? "Processing…"
                    : heardText
                      ? `“${heardText}”`
                      : voiceHint}
              </p>
            </div>
          ) : null}
          {activity ? (
            <p className="mt-4 text-xs text-stealth">{activity}</p>
          ) : null}
          {error ? <p className="mt-3 text-xs text-destructive">{error}</p> : null}
        </div>

        <form
          className="flex shrink-0 items-end gap-2 border-t border-white/10 p-3"
          onSubmit={(event) => {
            event.preventDefault();
            void send();
          }}
        >
          <textarea
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            rows={2}
            placeholder={
              listening
                ? voicePartial || "Listening… speak now"
                : "Ask anything — or hold the mic and talk"
            }
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                void send();
              }
            }}
            className="min-h-16 flex-1 resize-none rounded-lg border border-white/10 bg-black/30 px-3 py-2 text-sm outline-none focus:border-white/25"
          />
          <button
            type="button"
            aria-pressed={listening}
            aria-label={listening ? "Release to send" : "Hold to talk"}
            onPointerDown={(event) => {
              event.preventDefault();
              event.currentTarget.setPointerCapture(event.pointerId);
              void pressVoice();
            }}
            onPointerUp={() => void releaseVoice()}
            onPointerCancel={() => void releaseVoice()}
            onContextMenu={(event) => event.preventDefault()}
            className={`relative inline-flex h-12 min-w-12 items-center justify-center rounded-full border transition-colors ${
              listening
                ? "border-stealth/60 bg-stealth text-black"
                : "border-white/15 bg-white/10 text-foreground hover:bg-white/15"
            }`}
          >
            {listening ? (
              <span
                className="absolute inset-0 rounded-full bg-stealth/30"
                style={{ transform: `scale(${1 + Math.min(voiceLevel, 1) * 0.45})` }}
              />
            ) : null}
            <Mic className="relative size-4" />
          </button>
          <Button type="submit">
            <Send className="size-3.5" />
            Send
          </Button>
        </form>
        <p className="px-3 pb-3 text-[11px] text-muted-foreground">
          {voiceStatus(voicePhase, voiceHint, heardText)}
        </p>
      </section>

      <aside className="flex w-72 shrink-0 flex-col gap-4 overflow-y-auto border-l border-white/10 bg-black/25 p-4">
        <h2 className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Task</h2>
        {selected ? (
          <dl className="space-y-2 text-xs">
            <Row label="Status" value={(selected.status ?? "").replaceAll("_", " ")} />
            <Row label="Assistant" value={profiles.find((p) => p.id === selected.agentId)?.name ?? selected.agentId} />
            <Row label="Steps" value={`${(selected.plan ?? []).filter((s) => s.status === "DONE").length}/${(selected.plan ?? []).length}`} />
          </dl>
        ) : (
          <p className="text-xs text-muted-foreground">No task selected.</p>
        )}

        <div>
          <h2 className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Tools</h2>
          <ul className="mt-2 space-y-1 text-[11px] text-muted-foreground">
            {tools.slice(0, 10).map((tool) => (
              <li key={tool.name}>
                {tool.name}
                <span className="ml-1 opacity-60">{tool.riskLevel}</span>
              </li>
            ))}
          </ul>
        </div>

        {selected?.artifacts?.length ? (
          <div>
            <h2 className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Artifacts</h2>
            <ul className="mt-2 space-y-1 text-xs">
              {selected.artifacts.map((item) => (
                <li key={item.artifactId}>{item.path}</li>
              ))}
            </ul>
          </div>
        ) : null}
      </aside>
    </div>
  );
}

function CommandCenter({
  tasks,
  approvals,
  schedules,
  memories,
}: {
  tasks: AgentTask[];
  approvals: Approval[];
  schedules: number;
  memories: number;
}) {
  const active = useMemo(
    () =>
      tasks.filter((task) =>
        ["EXECUTING", "PLANNING", "UNDERSTANDING", "REFLECTING", "WAITING_FOR_TOOL"].includes(task.status),
      ),
    [tasks],
  );

  return (
    <div className="grid shrink-0 grid-cols-4 gap-2 border-b border-white/10 px-4 py-3 text-[11px]">
      <Stat icon={LoaderCircle} label="Active" value={String(active.length)} />
      <Stat icon={ShieldAlert} label="Approvals" value={String(approvals.length)} />
      <Stat icon={CalendarClock} label="Scheduled" value={String(schedules)} />
      <Stat icon={MemoryStick} label="Memories" value={String(memories)} />
    </div>
  );
}

function TaskCard({ task }: { task: AgentTask }) {
  const { pause, cancel, retry, selectTask } = useAgentStore();
  const status = task.status ?? "IDLE";
  const plan = task.plan ?? [];
  const errors = task.errors ?? [];
  const running = ["EXECUTING", "PLANNING", "UNDERSTANDING", "REFLECTING"].includes(status);

  return (
    <article className="mt-6 rounded-lg border border-white/10 bg-black/20 p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{task.goal}</h3>
          <p className="mt-1 text-[11px] uppercase tracking-wide text-muted-foreground">
            {status.replaceAll("_", " ")}
          </p>
        </div>
        <div className="flex gap-1">
          {running ? (
            <Button type="button" size="sm" variant="secondary" onClick={() => void pause(task.taskId)}>
              <Pause className="size-3" /> Pause
            </Button>
          ) : null}
          {status === "FAILED" ? (
            <Button type="button" size="sm" variant="secondary" onClick={() => void retry(task.taskId)}>
              Retry
            </Button>
          ) : null}
          {!["COMPLETED", "CANCELLED"].includes(status) ? (
            <Button type="button" size="sm" variant="ghost" onClick={() => void cancel(task.taskId)}>
              <Square className="size-3" /> Cancel
            </Button>
          ) : null}
        </div>
      </div>
      <ol className="mt-3 space-y-1.5 text-xs">
        {plan.map((step) => (
          <li key={step.stepId} className="flex items-start gap-2">
            {step.status === "DONE" ? (
              <Check className="mt-0.5 size-3 text-stealth" />
            ) : step.status === "RUNNING" ? (
              <LoaderCircle className="mt-0.5 size-3 animate-spin text-stealth" />
            ) : (
              <Circle className="mt-0.5 size-3 text-muted-foreground" />
            )}
            <span>{step.description}</span>
          </li>
        ))}
      </ol>
      {errors.length ? (
        <p className="mt-3 text-xs text-destructive">{errors[errors.length - 1]}</p>
      ) : null}
      {(task.toolCalls ?? []).length ? (
        <div className="mt-3 space-y-1.5 rounded-md border border-white/10 bg-black/30 p-2">
          <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">Actions</p>
          {(task.toolCalls ?? []).map((call, index) => (
            <p key={`${call.tool}-${index}`} className="text-[11px] leading-relaxed">
              <span className="text-stealth">{call.tool}</span>
              <span className="text-muted-foreground"> · {call.status}</span>
              {call.resultSummary ? <span> — {call.resultSummary}</span> : null}
            </p>
          ))}
        </div>
      ) : null}
      {task.result ? (
        <p className="selectable mt-3 whitespace-pre-wrap text-sm leading-relaxed">{task.result}</p>
      ) : running ? (
        <p className="mt-3 text-xs text-stealth">Working on it…</p>
      ) : null}
      <button
        type="button"
        className="mt-2 text-[11px] text-muted-foreground hover:text-foreground"
        onClick={() => selectTask(task.taskId)}
      >
        Inspect
      </button>
    </article>
  );
}

function ApprovalBanner({ approval }: { approval: Approval }) {
  const { approve, deny } = useAgentStore();
  return (
    <div className="shrink-0 border-b border-amber-500/30 bg-amber-500/10 px-4 py-3">
      <p className="flex items-center gap-2 text-[11px] font-medium uppercase tracking-[0.14em] text-amber-100">
        <ShieldAlert className="size-3.5" /> Action requires your approval
      </p>
      <p className="mt-2 text-sm">
        Coda wants to {approval.action.toLowerCase()} · {approval.target}
      </p>
      {approval.tool === "MEMORY_DELETE" && approval.arguments?.matches?.length ? (
        <ul className="mt-2 max-h-28 space-y-1 overflow-y-auto text-xs text-amber-50/90">
          {approval.arguments.matches.map((item) => (
            <li key={item.id}>
              <span className="uppercase text-amber-100/60">{item.source ?? "memory"}</span>
              {" — "}
              {item.content}
            </li>
          ))}
        </ul>
      ) : null}
      <p className="mt-1 text-xs text-amber-50/80">{approval.consequences}</p>
      <p className="mt-1 text-[11px] uppercase text-amber-100/70">Risk {approval.riskLevel}</p>
      <div className="mt-3 flex flex-wrap gap-2">
        <Button type="button" size="sm" onClick={() => void approve(approval.approvalId, "once")}>
          Approve once
        </Button>
        <Button type="button" size="sm" variant="secondary" onClick={() => void approve(approval.approvalId, "task")}>
          Approve for this task
        </Button>
        <Button type="button" size="sm" variant="ghost" onClick={() => void deny(approval.approvalId)}>
          <X className="size-3" /> Deny
        </Button>
      </div>
    </div>
  );
}

function Nav({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-md px-2 py-1 text-left text-xs ${active ? "bg-white/10 text-foreground" : "text-muted-foreground hover:text-foreground"}`}
    >
      {label}
    </button>
  );
}

function Stat({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof Bot;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-2 text-muted-foreground">
      <Icon className="size-3" />
      <span>
        {label} <span className="text-foreground">{value}</span>
      </span>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-3">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="text-right font-medium">{value}</dd>
    </div>
  );
}

function rowClass(active: boolean) {
  return `mb-1 w-full rounded-md px-2 py-1.5 text-left text-xs ${active ? "bg-white/10" : "hover:bg-white/5"}`;
}

function voiceStatus(
  phase: string,
  hint: string | null,
  heard: string | null,
): string {
  if (hint) return hint;
  if (phase === "recording") return "Listening… hold the mic or Ctrl+Space.";
  if (phase === "transcribing") return "Processing…";
  if (heard) return `You said: “${heard}”`;
  return "Hold the mic, speak, then release. Simple commands like “Open Apple Music” run immediately.";
}
