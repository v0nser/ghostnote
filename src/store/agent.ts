import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

import {
  AGENT_EVENTS,
  agentIpc,
  type AgentEvent,
  type AgentProfile,
  type AgentSnapshot,
  type AgentTask,
  type Approval,
  type Conversation,
  type MemoryItem,
  type ScheduledTask,
  type ScreenAskEvent,
  type SetupStatus,
  type ToolDefinition,
} from "@/lib/ipc/agent";
import { CAPTURE_EVENTS, type LevelEvent, type TranscriptSegment, type VadEvent } from "@/lib/ipc/capture";
import { describeIpcError } from "@/lib/ipc/stealth";
import { log } from "@/lib/logger";

export type AppView = "meeting" | "agent" | "setup" | "audit";
export type AgentPane = "assistants" | "conversations" | "tasks" | "scheduled" | "memory";
export type VoicePhase =
  | "idle"
  | "recording"
  | "stopping"
  | "transcribing"
  | "final"
  | "error"
  | "cancelled";

export interface ScreenAskState {
  open: boolean;
  phase: string;
  question: string;
  text: string;
  error: string | null;
}

interface AgentStore {
  view: AppView;
  pane: AgentPane;
  ready: boolean;
  error: string | null;
  activity: string | null;
  listening: boolean;
  voicePhase: VoicePhase;
  voiceLevel: number;
  voiceHint: string | null;
  voicePartial: string;
  heardText: string | null;
  screenAsk: ScreenAskState;

  profiles: AgentProfile[];
  conversations: Conversation[];
  tasks: AgentTask[];
  approvals: Approval[];
  memories: MemoryItem[];
  schedules: ScheduledTask[];
  tools: ToolDefinition[];
  setup: SetupStatus | null;
  download: { received: number; total: number } | null;

  activeAgentId: string;
  activeConversationId: string | null;
  selectedTaskId: string | null;
  draft: string;

  setView: (view: AppView) => void;
  setPane: (pane: AgentPane) => void;
  setDraft: (draft: string) => void;
  selectAgent: (id: string) => void;
  selectConversation: (id: string) => void;
  selectTask: (id: string | null) => void;

  init: () => Promise<void>;
  refresh: () => Promise<void>;
  send: () => Promise<void>;
  startVoice: () => Promise<void>;
  stopVoice: (submit?: boolean) => Promise<void>;
  pressVoice: () => Promise<void>;
  releaseVoice: () => Promise<void>;
  approve: (id: string, scope?: string) => Promise<void>;
  deny: (id: string) => Promise<void>;
  pause: (id: string) => Promise<void>;
  cancel: (id: string) => Promise<void>;
  retry: (id: string) => Promise<void>;
  remember: (content: string) => Promise<void>;
  forget: (id: string) => Promise<void>;
  createFollowUp: (goal: string) => Promise<void>;
  downloadWhisper: () => Promise<void>;
  setAutostart: (enabled: boolean) => Promise<void>;
  applyScreenAsk: (event: ScreenAskEvent) => void;
  dismissScreenAsk: () => void;
}

let unlisteners: UnlistenFn[] = [];
let whisperUnlisten: UnlistenFn[] = [];
let silenceTimer: number | null = null;
let maxTimer: number | null = null;
let pressStartedAt = 0;
let tapListening = false;
let endingVoice = false;

function applySnapshot(snapshot: AgentSnapshot): Partial<AgentStore> {
  return {
    profiles: snapshot.profiles,
    conversations: snapshot.conversations,
    tasks: snapshot.tasks,
    approvals: snapshot.approvals,
    memories: snapshot.memories,
    schedules: snapshot.schedules,
  };
}

export const useAgentStore = create<AgentStore>((set, get) => ({
  view: "meeting",
  pane: "conversations",
  ready: false,
  error: null,
  activity: null,
  listening: false,
  voicePhase: "idle",
  voiceLevel: 0,
  voiceHint: null,
  voicePartial: "",
  heardText: null,
  screenAsk: { open: false, phase: "idle", question: "", text: "", error: null },

  profiles: [],
  conversations: [],
  tasks: [],
  approvals: [],
  memories: [],
  schedules: [],
  tools: [],
  setup: null,
  download: null,

  activeAgentId: "personal",
  activeConversationId: null,
  selectedTaskId: null,
  draft: "",

  setView: (view) => set({ view }),
  setPane: (pane) => set({ pane }),
  setDraft: (draft) => set({ draft }),
  selectAgent: (id) => set({ activeAgentId: id }),
  selectConversation: (id) => set({ activeConversationId: id }),
  selectTask: (id) => set({ selectedTaskId: id }),

  init: async () => {
    if (unlisteners.length > 0) return;
    unlisteners = [() => {}];
    await get().refresh();
    try {
      unlisteners = await Promise.all([
        listen<AgentEvent>(AGENT_EVENTS.event, ({ payload }) => {
          const summary =
            typeof payload.data?.summary === "string"
              ? payload.data.summary
              : payload.event.replaceAll(".", " ");
          set({ activity: summary });
          if (payload.taskId) set({ selectedTaskId: payload.taskId });
          void get().refresh();
        }),
        listen<{ received: number; total: number }>(AGENT_EVENTS.download, ({ payload }) => {
          set({ download: payload });
        }),
        listen<ScreenAskEvent>(AGENT_EVENTS.screenAsk, ({ payload }) => {
          get().applyScreenAsk(payload);
        }),
      ]);
    } catch {
      unlisteners = [() => {}];
    }
  },

  refresh: async () => {
    try {
      const [snapshot, tools, setup] = await Promise.all([
        agentIpc.overview(),
        agentIpc.tools(),
        agentIpc.setupStatus(),
      ]);
      const firstRun = setup.firstRun && get().view === "meeting" && !get().ready;
      set({
        ...applySnapshot(snapshot),
        tools,
        setup,
        ready: true,
        error: null,
        view: firstRun ? "setup" : get().view,
        activeConversationId:
          get().activeConversationId ?? snapshot.conversations[0]?.id ?? null,
      });
    } catch (error) {
      log.error("could not load the agent workspace");
      set({ error: describeIpcError(error), ready: true });
    }
  },

  send: async () => {
    if (get().listening || get().voicePhase === "transcribing") return;
    const goal = get().draft.trim();
    if (!goal) return;
    set({ draft: "", error: null, activity: "Understanding the request" });
    try {
      const task = await agentIpc.createTask(
        goal,
        get().activeAgentId,
        get().activeConversationId ?? undefined,
      );
      set({
        selectedTaskId: task.taskId,
        activeConversationId: task.conversationId,
        view: "agent",
      });
      await get().refresh();
    } catch (error) {
      set({ error: describeIpcError(error) });
    }
  },

  pressVoice: async () => {
    pressStartedAt = Date.now();
    if (get().listening) {
      tapListening = false;
      await get().stopVoice(true);
      return;
    }
    tapListening = false;
    await get().startVoice();
  },

  releaseVoice: async () => {
    if (!get().listening) return;
    const heldMs = Date.now() - pressStartedAt;
    if (heldMs < 220) {
      tapListening = true;
      set({ voiceHint: "Listening… speak, then tap the mic to send" });
      return;
    }
    await get().stopVoice(true);
  },

  startVoice: async () => {
    if (get().listening) return;
    clearVoiceTimers();
    await abortHardwareVoice();

    // Never use Web Speech / SFSpeechRecognizer. In `tauri dev` the process is
    // a bare binary with no Info.plist, and macOS TCC aborts the app if that
    // API is touched. Whisper stays on-device and does not hit that path.
    if (!get().setup?.whisperInstalled) {
      set({
        listening: false,
        voicePhase: "error",
        voiceLevel: 0,
        voiceHint: "Couldn't understand that. Try again.",
        voicePartial: "",
        activity: null,
        error: "Download the speech model in Setup, then hold the mic to talk.",
        view: "setup",
      });
      return;
    }

    set({
      listening: true,
      voicePhase: "recording",
      voiceLevel: 0,
      voiceHint: "Listening…",
      voicePartial: "",
      heardText: null,
      error: null,
      activity: "Listening",
      view: "agent",
    });

    const onHeard = () => {
      if (!tapListening) return;
      armSilence(() => {
        void get().stopVoice(true);
      });
    };

    try {
      await startWhisperVoice(set, onHeard);
      maxTimer = window.setTimeout(() => {
        void get().stopVoice(true);
      }, 20_000);
    } catch (error) {
      await abortHardwareVoice();
      const message = describeIpcError(error);
      const needsSetup = /speech model is not installed/i.test(message);
      set({
        listening: false,
        voicePhase: "error",
        voiceLevel: 0,
        voiceHint: "Couldn't understand that. Try again.",
        voicePartial: "",
        activity: null,
        error: needsSetup
          ? "Download the speech model in Setup, then hold the mic to talk."
          : message,
        view: needsSetup ? "setup" : get().view,
      });
    }
  },

  stopVoice: async (submit = true) => {
    if (!get().listening || endingVoice) return;
    endingVoice = true;
    clearVoiceTimers();
    tapListening = false;
    set({
      voicePhase: submit ? "transcribing" : "cancelled",
      voiceHint: submit ? "Processing…" : null,
      activity: submit ? "Processing" : null,
    });

    let spoken = "";
    try {
      spoken = (await agentIpc.finishVoice()).trim();
      for (const stop of whisperUnlisten) stop();
      whisperUnlisten = [];
    } catch (error) {
      await abortHardwareVoice();
      endingVoice = false;
      set({
        listening: false,
        voicePhase: "error",
        voiceLevel: 0,
        voiceHint: "Couldn't understand that. Try again.",
        voicePartial: "",
        activity: null,
        error: describeIpcError(error),
      });
      return;
    }

    endingVoice = false;
    set({
      listening: false,
      voiceLevel: 0,
      voicePartial: "",
    });

    if (!submit) {
      set({ voicePhase: "cancelled", voiceHint: null, activity: null });
      return;
    }

    if (spoken.length < 2) {
      set({
        voicePhase: "error",
        heardText: null,
        voiceHint: "Couldn't understand that. Try again.",
        activity: null,
        error: "Couldn't understand that. Try again.",
      });
      return;
    }

    set({
      voicePhase: "final",
      heardText: spoken,
      draft: spoken,
      voiceHint: `You said: “${spoken}”`,
      activity: "Understanding…",
      error: null,
    });
    await get().send();
  },

  approve: async (id, scope) => {
    try {
      await agentIpc.approve(id, scope);
      await get().refresh();
    } catch (error) {
      set({ error: describeIpcError(error) });
    }
  },

  deny: async (id) => {
    try {
      await agentIpc.deny(id);
      await get().refresh();
    } catch (error) {
      set({ error: describeIpcError(error) });
    }
  },

  pause: async (id) => {
    await agentIpc.pause(id);
    await get().refresh();
  },

  cancel: async (id) => {
    await agentIpc.cancel(id);
    await get().refresh();
  },

  retry: async (id) => {
    await agentIpc.retry(id);
    await get().refresh();
  },

  remember: async (content) => {
    await agentIpc.remember(content);
    await get().refresh();
  },

  forget: async (id) => {
    await agentIpc.forget(id);
    await get().refresh();
  },

  createFollowUp: async (goal) => {
    const task = await agentIpc.createTask(goal, "personal");
    set({ view: "agent", selectedTaskId: task.taskId });
    await get().refresh();
  },

  downloadWhisper: async () => {
    set({ error: null });
    try {
      const setup = await agentIpc.downloadWhisper();
      set({ setup, download: null });
    } catch (error) {
      set({ error: describeIpcError(error) });
    }
  },

  setAutostart: async (enabled) => {
    set({ error: null });
    try {
      const setup = await agentIpc.setAutostart(enabled);
      set({ setup });
    } catch (error) {
      set({ error: describeIpcError(error) });
    }
  },

  applyScreenAsk: (event) => {
    set({
      screenAsk: {
        open: true,
        phase: event.phase,
        question: event.question,
        text: event.text,
        error: event.error,
      },
    });
  },

  dismissScreenAsk: () => {
    set({
      screenAsk: { open: false, phase: "idle", question: "", text: "", error: null },
    });
  },
}));

function clearVoiceTimers() {
  if (silenceTimer !== null) window.clearTimeout(silenceTimer);
  if (maxTimer !== null) window.clearTimeout(maxTimer);
  silenceTimer = null;
  maxTimer = null;
}

function armSilence(onQuiet: () => void) {
  if (silenceTimer !== null) window.clearTimeout(silenceTimer);
  silenceTimer = window.setTimeout(onQuiet, 2200);
}

async function abortHardwareVoice() {
  for (const stop of whisperUnlisten) stop();
  whisperUnlisten = [];
  try {
    await agentIpc.abortVoice();
  } catch {
    /* not in a hardware session */
  }
}

async function startWhisperVoice(
  set: (partial: Partial<AgentStore>) => void,
  onHeard: () => void,
) {
  await agentIpc.startVoice();
  whisperUnlisten = await Promise.all([
    listen<TranscriptSegment>(CAPTURE_EVENTS.segment, ({ payload }) => {
      if (payload.speaker !== "you") return;
      const next = payload.text.trim();
      if (!next) return;
      set({ voicePartial: next, voiceHint: "Listening…" });
      onHeard();
    }),
    listen<LevelEvent>(CAPTURE_EVENTS.level, ({ payload }) => {
      if (payload.speaker !== "you") return;
      set({ voiceLevel: payload.peak });
      if (payload.peak > 0.12) onHeard();
    }),
    listen<VadEvent>(CAPTURE_EVENTS.vad, ({ payload }) => {
      if (payload.speaker !== "you") return;
      if (payload.speaking) onHeard();
    }),
  ]);
}
