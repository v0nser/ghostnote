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

/** Mirrors `meeting::ActionItem`. */
export interface ActionItem {
  owner: string | null;
  task: string;
  due: string | null;
}

/** Mirrors `meeting::Nudge`. */
export interface MeetingNudge {
  kind: string;
  title: string;
  detail: string;
}

/** Mirrors `meeting::Draft`. */
export interface MeetingDraft {
  id: string;
  meetingId: string;
  kind: string;
  title: string;
  body: string;
  when: string | null;
  to: string | null;
  status: string;
}

/** Mirrors `meeting::MeetingSnapshot`. */
export interface MeetingSnapshot {
  items: Array<{
    id: string;
    meetingId: string;
    owner: string | null;
    task: string;
    due: string | null;
    status: string;
    createdAt: number;
  }>;
  topics: Array<{ label: string; lastSeen: number; count: number }>;
  people: Array<{ name: string; lastSeen: number; count: number }>;
  drafts: MeetingDraft[];
  nudges: MeetingNudge[];
}

/** Mirrors `ollama::MeetingSummary`. */
export interface MeetingSummary {
  text: string;
  model: string;
  actionItems?: ActionItem[];
  topics?: string[];
  people?: string[];
  nudges?: MeetingNudge[];
  drafts?: MeetingDraft[];
  extractModel?: string | null;
  extractError?: string | null;
}

export const COACH_EVENTS = {
  points: "coda://talking-points",
  status: "coda://coach-status",
} as const;

export const coachIpc = {
  status: () => invoke<CoachAvailability>("coach_status"),
  summarize: () => invoke<MeetingSummary>("summarize_meeting"),
  snapshot: () => invoke<MeetingSnapshot>("meeting_snapshot"),
  decideDraft: (draftId: string, approve: boolean) =>
    invoke<MeetingDraft>("meeting_decide_draft", { draftId, approve }),
};
