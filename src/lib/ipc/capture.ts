import { invoke } from "@tauri-apps/api/core";

/** Mirrors `audio::types::Speaker`. */
export type Speaker = "you" | "participant";

/** Mirrors `audio::types::CaptureStatus`. */
export interface CaptureStatus {
  running: boolean;
  microphone: boolean;
  systemAudio: boolean;
  /** Set when capture is running microphone-only because system audio failed. */
  systemAudioError: string | null;
  elapsedMs: number;
}

/** Mirrors `transcribe::TranscriptSegment`. */
export interface TranscriptSegment {
  id: number;
  speaker: Speaker;
  speakerLabel: string;
  text: string;
  startMs: number;
  endMs: number;
  provisional: boolean;
}

/** Mirrors `audio::types::LevelEvent`. */
export interface LevelEvent {
  speaker: Speaker;
  peak: number;
}

/** Mirrors `audio::types::VadEvent`. */
export interface VadEvent {
  speaker: Speaker;
  speaking: boolean;
}

/** Mirrors `audio::types::AudioDevice`. */
export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

/** Mirrors `audio::commands::SystemAudioSupport`. */
export interface SystemAudioSupport {
  supported: boolean;
  requiredPermission: string | null;
}

/** Mirrors `session::SessionStatus`. */
export interface SessionStatus {
  capture: CaptureStatus;
  modelInstalled: boolean;
  coach: {
    available: boolean;
    model: string | null;
  };
}

export interface CaptureOptions {
  microphoneDeviceId?: string | null;
  captureSystemAudio?: boolean;
}

export const CAPTURE_EVENTS = {
  segment: "ghostnote://transcript-segment",
  transcriptError: "ghostnote://transcript-error",
  level: "ghostnote://audio-level",
  vad: "ghostnote://vad",
} as const;

export const captureIpc = {
  sessionStatus: () => invoke<SessionStatus>("session_status"),
  start: (options?: CaptureOptions) => invoke<CaptureStatus>("start_capture", { options }),
  stop: () => invoke<CaptureStatus>("stop_capture"),
  inputDevices: () => invoke<AudioDevice[]>("list_input_devices"),
  systemAudioSupport: () => invoke<SystemAudioSupport>("system_audio_support"),
};
