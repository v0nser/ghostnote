import { invoke } from "@tauri-apps/api/core";

/** Mirrors `stealth::StealthStatus` in the Rust backend. */
export interface StealthStatus {
  /** Stealth Mode is fully engaged. */
  enabled: boolean;
  /** The OS has confirmed the window is excluded from capture. */
  captureExcluded: boolean;
  /** The window is wearing the compact floating pill shell. */
  pillMode: boolean;
  /** Outgoing notifications are being dropped. */
  notificationsSuppressed: boolean;
  /** This OS exposes a capture-exclusion primitive at all. */
  platformSupported: boolean;
  /** Name of the OS primitive in use, surfaced so the user can verify it. */
  backend: string;
}

export const stealthIpc = {
  status: () => invoke<StealthStatus>("stealth_status"),
  setEnabled: (enabled: boolean) => invoke<StealthStatus>("set_stealth_enabled", { enabled }),
  toggle: () => invoke<StealthStatus>("toggle_stealth"),
  verify: () => invoke<boolean>("verify_capture_exclusion"),
  setPillExpanded: (expanded: boolean) =>
    invoke<StealthStatus>("set_pill_expanded", { expanded }),
};

/**
 * Tauri rejects with whatever the command returned, which for us is the
 * `Display` output of `StealthError`. Normalise it to a string without ever
 * echoing an unknown payload, which could contain user content.
 */
export function describeIpcError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "The operating system rejected the request.";
}
