import { invoke } from "@tauri-apps/api/core";

/** Mirrors `ollama::TalkingPoints`. */
export interface TalkingPoints {
  id: number;
  cue: string;
  question: string;
  answer: string;
  model: string;
}

/** Mirrors `ollama::client::CoachStatus`. */
export interface CoachAvailability {
  available: boolean;
  model: string | null;
}

export type CoachPhase = "idle" | "processing" | "writing";

/** Mirrors `ollama::LiveCoachStatus`. */
export interface LiveCoachStatus {
  available: boolean;
  generating: boolean;
  phase: CoachPhase;
  model: string | null;
  message: string | null;
  pendingCue: string | null;
}

/** Mirrors `ollama::MeetingSummary`. */
export interface MeetingSummary {
  text: string;
  model: string;
}

export const COACH_EVENTS = {
  points: "ghostnote://talking-points",
  status: "ghostnote://coach-status",
} as const;

export const coachIpc = {
  status: () => invoke<CoachAvailability>("coach_status"),
  summarize: () => invoke<MeetingSummary>("summarize_meeting"),
};
