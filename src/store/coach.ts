import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

import { CAPTURE_EVENTS, type VadEvent } from "@/lib/ipc/capture";
import {
  COACH_EVENTS,
  coachIpc,
  type CoachPhase,
  type LiveCoachStatus,
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

  init: () => Promise<void>;
  summarize: () => Promise<void>;
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

  init: async () => {
    if (unlisteners.length > 0) return;
    unlisteners = [() => {}];

    try {
      const status = await coachIpc.status();
      set({ available: status.available, model: status.model });
    } catch {
      log.error("could not reach the local language model");
      set({ available: false });
    }

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
  },

  summarize: async () => {
    if (get().summarizing) return;
    set({ summarizing: true, message: null });
    try {
      const result: MeetingSummary = await coachIpc.summarize();
      set({ summary: result.text, summarizing: false });
    } catch (error) {
      log.error("could not summarize the meeting");
      set({
        summarizing: false,
        message: describeIpcError(error),
      });
    }
  },

  clearSummary: () => set({ summary: null }),
}));
