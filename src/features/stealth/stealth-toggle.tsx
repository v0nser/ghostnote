import { EyeOff, ShieldAlert } from "lucide-react";

import { Switch } from "@/components/ui/switch";
import { useStealthStore } from "@/store/stealth";
import { cn } from "@/lib/utils";

/**
 * The single most important control in the app.
 *
 * Two rules govern how it renders:
 *
 * 1. It reflects the **backend's** status, never optimistic local state. If
 *    the OS call fails the switch stays off, because a switch that says "on"
 *    while the window is still capturable is actively dangerous.
 * 2. On a platform without capture exclusion it is disabled outright rather
 *    than offering protection it cannot deliver.
 */
export function StealthToggle() {
  const { status, pending, setEnabled } = useStealthStore();

  const supported = status?.platformSupported ?? false;
  const enabled = status?.enabled ?? false;

  return (
    <div
      className={cn(
        "flex shrink-0 items-center gap-3 rounded-full border px-3 py-1.5 transition-colors",
        enabled ? "border-stealth/40 bg-stealth/10" : "border-border bg-white/3",
      )}
    >
      <EyeOff
        className={cn("size-3.5", enabled ? "text-stealth" : "text-muted-foreground")}
        aria-hidden
      />
      <span
        className={cn(
          "text-xs font-medium tracking-wide whitespace-nowrap",
          enabled ? "text-stealth" : "text-muted-foreground",
        )}
      >
        {enabled ? "Hidden from capture" : "Visible"}
      </span>
      <Switch
        checked={enabled}
        disabled={pending || !supported}
        onCheckedChange={(next) => void setEnabled(next)}
        aria-label="Stealth Mode"
      />
    </div>
  );
}

/**
 * Failure notices live in their own full-width strip rather than beside the
 * toggle. An OS error string is arbitrarily long, and letting it share a row
 * with the toggle would push the header's contents off-screen.
 */
export function StealthBanner() {
  const { status, error } = useStealthStore();

  // An unsupported platform outranks a transient error: it is permanent, and
  // it is the one thing the user must know before they share their screen.
  const message =
    status && !status.platformSupported
      ? "Capture exclusion is unavailable on this platform — this window will still appear in screen shares."
      : error;

  if (!message) return null;

  return (
    <div className="flex shrink-0 items-start gap-2 border-b border-destructive/25 bg-destructive/10 px-4 py-2">
      <ShieldAlert className="mt-0.5 size-3.5 shrink-0 text-destructive" aria-hidden />
      <p className="min-w-0 text-xs leading-relaxed text-destructive">{message}</p>
    </div>
  );
}
