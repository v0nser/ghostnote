import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

import {
  CAPTURE_EVENTS,
  captureIpc,
  type AudioDevice,
  type CaptureStatus,
  type LevelEvent,
  type Speaker,
  type SystemAudioSupport,
  type TranscriptSegment,
} from "@/lib/ipc/capture";
import { describeIpcError } from "@/lib/ipc/stealth";
import { log } from "@/lib/logger";
import { useCoachStore } from "@/store/coach";

const IDLE_STATUS: CaptureStatus = {
  running: false,
  microphone: false,
  systemAudio: false,
  systemAudioError: null,
  elapsedMs: 0,
};

interface CaptureStore {
  status: CaptureStatus;
  modelInstalled: boolean;
  devices: AudioDevice[];
  systemAudioSupport: SystemAudioSupport | null;

  segments: TranscriptSegment[];
  levels: Record<Speaker, number>;

  pending: boolean;
  error: string | null;

  init: () => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  clearTranscript: () => void;
}

let unlisteners: UnlistenFn[] = [];

export const useCaptureStore = create<CaptureStore>((set, get) => ({
  status: IDLE_STATUS,
  modelInstalled: false,
  devices: [],
  systemAudioSupport: null,

  segments: [],
  levels: { you: 0, participant: 0 },

  pending: false,
  error: null,

  /**
   * Reads the backend's state and subscribes to the capture event streams.
   *
   * Safe to call more than once: React's StrictMode mounts effects twice in
   * development, and without the guard the transcript would receive every
   * segment twice.
   */
  init: async () => {
    if (unlisteners.length > 0) return;

    // Claim the slot before the first await so a concurrent call cannot race
    // past the guard above and register a second set of listeners.
    unlisteners = [() => {}];

    try {
      const [session, devices, systemAudioSupport] = await Promise.all([
        captureIpc.sessionStatus(),
        captureIpc.inputDevices(),
        captureIpc.systemAudioSupport(),
      ]);

      set({
        status: session.capture,
        modelInstalled: session.modelInstalled,
        devices,
        systemAudioSupport,
      });
    } catch (error) {
      log.error("could not read capture state");
      set({ error: describeIpcError(error) });
    }

    unlisteners = await Promise.all([
      listen<TranscriptSegment>(CAPTURE_EVENTS.segment, ({ payload }) => {
        set((state) => {
          const last = state.segments[state.segments.length - 1];
          const replaceLast =
            last &&
            last.speaker === payload.speaker &&
            (last.provisional || payload.provisional);

          if (replaceLast) {
            return { segments: [...state.segments.slice(0, -1), payload] };
          }
          return { segments: [...state.segments, payload] };
        });
      }),

      listen<LevelEvent>(CAPTURE_EVENTS.level, ({ payload }) => {
        set((state) => ({
          levels: { ...state.levels, [payload.speaker]: payload.peak },
        }));
      }),

      listen<{ message: string }>(CAPTURE_EVENTS.transcriptError, ({ payload }) => {
        log.error("transcription reported a failure");
        set({ error: payload.message });
      }),
    ]);
  },

  start: async () => {
    if (get().pending) return;
    set({ pending: true });

    try {
      useCoachStore.getState().clearSummary();
      set({ status: await captureIpc.start(), error: null, segments: [] });
    } catch (error) {
      log.error("could not start capture");
      set({ error: describeIpcError(error) });
    } finally {
      set({ pending: false });
    }
  },

  stop: async () => {
    if (get().pending) return;
    set({ pending: true });

    try {
      // Levels are reset explicitly: the meter stops receiving events the
      // moment capture ends, so its last value would otherwise stay frozen
      // on screen suggesting audio is still being picked up.
      set({ status: await captureIpc.stop(), levels: { you: 0, participant: 0 } });
    } catch (error) {
      log.error("could not stop capture");
      set({ error: describeIpcError(error) });
    } finally {
      set({ pending: false });
    }
  },

  clearTranscript: () => set({ segments: [], error: null }),
}));
