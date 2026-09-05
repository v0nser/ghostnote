import { create } from "zustand";

import { describeIpcError, stealthIpc, type StealthStatus } from "@/lib/ipc/stealth";
import { log } from "@/lib/logger";

interface StealthStore {
  status: StealthStatus | null;
  /** A toggle is in flight; the switch is disabled until the OS answers. */
  pending: boolean;
  /** Last failure, shown inline. Cleared on the next successful call. */
  error: string | null;

  refresh: () => Promise<void>;
  setEnabled: (enabled: boolean) => Promise<void>;
  toggle: () => Promise<void>;
}

export const useStealthStore = create<StealthStore>((set, get) => {
  /**
   * Every mutation funnels through here so the pending flag and the error
   * string can never drift out of sync with the backend. On failure the
   * status is deliberately left untouched: the Rust side rolls the window
   * back, so the last known-good status is still accurate.
   */
  const run = async (action: () => Promise<StealthStatus>) => {
    if (get().pending) return;
    set({ pending: true });

    try {
      set({ status: await action(), error: null });
    } catch (error) {
      log.error("stealth toggle rejected by the backend");
      set({ error: describeIpcError(error) });
    } finally {
      set({ pending: false });
    }
  };

  return {
    status: null,
    pending: false,
    error: null,

    refresh: async () => {
      try {
        set({ status: await stealthIpc.status() });
      } catch (error) {
        log.error("could not read stealth status");
        set({ error: describeIpcError(error) });
      }
    },

    setEnabled: (enabled) => run(() => stealthIpc.setEnabled(enabled)),
    toggle: () => run(() => stealthIpc.toggle()),
  };
});
