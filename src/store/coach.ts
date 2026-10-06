import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

import { CAPTURE_EVENTS, type VadEvent } from "@/lib/ipc/capture";
import {
  COACH_EVENTS,
  coachIpc,
  type ActionItem,
  type CoachPhase,
  type LiveCoachStatus,
  type MeetingDraft,
  type MeetingNudge,
  type MeetingSummary,
  type TalkingPoints,
} from "@/lib/ipc/coach";
import { describeIpcError } from "@/lib/ipc/stealth";
import { log } from "@/lib/logger";

interface CoachStore {
  available: boolean;
  generating: boolean;
  phase: CoachPhase;
  model: string | null;
  message: string | null;
  pendingCue: string | null;
  suggestion: TalkingPoints | null;
  summary: string | null;
  summarizing: boolean;
  actionItems: ActionItem[];
  topics: string[];
  people: string[];
  nudges: MeetingNudge[];
  drafts: MeetingDraft[];
  extractError: string | null;

  init: () => Promise<void>;
  summarize: () => Promise<void>;
  decideDraft: (draftId: string, approve: boolean) => Promise<void>;
  clearSummary: () => void;
}

let unlisteners: UnlistenFn[] = [];

export const useCoachStore = create<CoachStore>((set, get) => ({
  available: false,
  generating: false,
  phase: "idle",
  model: null,
  message: null,
  pendingCue: null,
  suggestion: null,
  summary: null,
  summarizing: false,
  actionItems: [],
  topics: [],
  people: [],
  nudges: [],
  drafts: [],
  extractError: null,

  init: async () => {
    if (unlisteners.length > 0) return;
    unlisteners = [() => {}];

    try {
      const [status, snapshot] = await Promise.all([
        coachIpc.status(),
        coachIpc.snapshot().catch(() => null),
      ]);
      set({
        available: status.available,
        model: status.model,
        nudges: snapshot?.nudges ?? [],
        drafts: snapshot?.drafts ?? [],
        topics: snapshot?.topics.map((topic) => topic.label) ?? [],
        people: snapshot?.people.map((person) => person.name) ?? [],
      });
    } catch {
      log.error("could not reach the local language model");
      set({ available: false });
    }

    try {
      unlisteners = await Promise.all([
        listen<TalkingPoints | null>(COACH_EVENTS.points, ({ payload }) => {
          set({ suggestion: payload });
        }),
        listen<LiveCoachStatus>(COACH_EVENTS.status, ({ payload }) => {
          set({
            available: payload.available,
            generating: payload.generating,
            phase: payload.phase ?? (payload.generating ? "writing" : "idle"),
            model: payload.model ?? null,
            message: payload.message,
            pendingCue: payload.pendingCue,
          });
        }),
        listen<VadEvent>(CAPTURE_EVENTS.vad, ({ payload }) => {
          if (payload.speaking || payload.speaker !== "participant") return;
          if (!get().available) return;
          set({
            generating: true,
            phase: "processing",
            suggestion: null,
            pendingCue: null,
            message: null,
          });
        }),
      ]);
    } catch {
      unlisteners = [() => {}];
    }
  },

  summarize: async () => {
    if (get().summarizing) return;
    set({ summarizing: true, message: null });
    try {
      const result: MeetingSummary = await coachIpc.summarize();
      set({
        summary: result.text,
        summarizing: false,
        actionItems: result.actionItems ?? [],
        topics: result.topics ?? [],
        people: result.people ?? [],
        nudges: result.nudges ?? [],
        drafts: result.drafts ?? [],
        extractError: result.extractError ?? null,
      });
    } catch (error) {
      log.error("could not summarize the meeting");
      set({
        summarizing: false,
        message: describeIpcError(error),
      });
    }
  },

  decideDraft: async (draftId, approve) => {
    try {
      const decided = await coachIpc.decideDraft(draftId, approve);
      set({
        drafts: get().drafts.map((draft) => (draft.id === decided.id ? decided : draft)),
      });
    } catch (error) {
      log.error("could not decide meeting draft");
      set({ message: describeIpcError(error) });
    }
  },

  clearSummary: () =>
    set({
      summary: null,
      actionItems: [],
      extractError: null,
    }),
}));
