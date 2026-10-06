import { useEffect } from "react";

import { useAgentStore } from "@/store/agent";

export function ScreenAskOverlay() {
  const screenAsk = useAgentStore((state) => state.screenAsk);
  const dismissScreenAsk = useAgentStore((state) => state.dismissScreenAsk);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && screenAsk.open) {
        event.preventDefault();
        dismissScreenAsk();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [screenAsk.open, dismissScreenAsk]);

  if (!screenAsk.open) return null;

  const body =
    screenAsk.error ??
    (screenAsk.text.trim() ? screenAsk.text : "Looking at your screen…");

  return (
    <div className="pointer-events-auto fixed bottom-4 right-4 z-50 w-[min(360px,calc(100%-2rem))] rounded-xl border border-white/15 bg-[#161618] p-3 shadow-2xl">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-[10px] uppercase tracking-[0.16em] text-muted-foreground">On screen</p>
          <p className="mt-0.5 truncate text-xs text-foreground/80">{screenAsk.question}</p>
        </div>
        <button
          type="button"
          className="rounded-md px-1.5 py-0.5 text-[11px] text-muted-foreground hover:bg-white/10 hover:text-foreground"
          onClick={() => dismissScreenAsk()}
        >
          Esc
        </button>
      </div>
      <p
        className={`mt-2 max-h-48 overflow-y-auto whitespace-pre-wrap text-sm leading-relaxed ${
          screenAsk.error ? "text-destructive" : "text-foreground"
        }`}
      >
        {body}
      </p>
    </div>
  );
}
