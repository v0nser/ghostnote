import { Minus, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { cn } from "@/lib/utils";

/**
 * The window is undecorated so the app can shed its titlebar in pill mode.
 * These are the replacement controls.
 *
 * Deliberately no `title` attributes and no hover-revealed labels: a native
 * tooltip is drawn by the OS *outside* our window, which means capture
 * exclusion does not cover it. A tooltip appearing over a shared screen would
 * defeat the entire product.
 */
export function WindowControls({ className }: { className?: string }) {
  // Resolved lazily inside the handlers: `getCurrentWindow()` reads Tauri
  // internals off `window`, which do not exist when the same bundle is opened
  // in a plain browser during frontend-only development.
  const appWindow = () => getCurrentWindow();

  return (
    <div className={cn("flex items-center gap-1", className)}>
      <button
        type="button"
        aria-label="Minimise"
        onClick={() => void appWindow().minimize()}
        className="flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-white/8 hover:text-foreground"
      >
        <Minus className="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Close"
        onClick={() => void appWindow().close()}
        className="flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
      >
        <X className="size-3.5" />
      </button>
    </div>
  );
}

/**
 * Marks a region as draggable. `data-tauri-drag-region` is handled natively by
 * the webview, so nested interactive elements must opt out with
 * `data-tauri-drag-region="false"` or they become un-clickable.
 */
export function DragRegion({
  className,
  children,
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <div data-tauri-drag-region className={className}>
      {children}
    </div>
  );
}
