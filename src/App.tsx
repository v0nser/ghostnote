import { useEffect, type ReactNode } from "react";

import { StealthPill } from "@/features/stealth/stealth-pill";
import { SetupPanel } from "@/features/setup/setup-panel";
import { useAgentStore } from "@/store/agent";
import { useCaptureStore } from "@/store/capture";
import { useCoachStore } from "@/store/coach";
import { useStealthStore } from "@/store/stealth";
import { Dashboard } from "@/views/dashboard";
import { AgentWorkspace } from "@/views/workspace";
import { AuditView } from "@/views/audit";
import { ScreenAskOverlay } from "@/features/screen/screen-overlay";
import { AppErrorBoundary } from "@/components/error-boundary";
import { DragRegion, WindowControls } from "@/components/window-chrome";

export default function App() {
  const status = useStealthStore((state) => state.status);
  const refresh = useStealthStore((state) => state.refresh);
  const initCapture = useCaptureStore((state) => state.init);
  const initCoach = useCoachStore((state) => state.init);
  const initAgent = useAgentStore((state) => state.init);
  const view = useAgentStore((state) => state.view);
  const setView = useAgentStore((state) => state.setView);
  const setup = useAgentStore((state) => state.setup);

  useEffect(() => {
    void refresh();
    void initCapture();
    void initCoach();
    void initAgent();
  }, [refresh, initCapture, initCoach, initAgent]);

  if (status?.pillMode) {
    return (
      <>
        <StealthPill />
        <ScreenAskOverlay />
      </>
    );
  }

  return (
    <div className="readable relative flex h-full w-full flex-col overflow-hidden rounded-xl border border-white/20 bg-[#121214]">
      <DragRegion className="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-white/10 bg-black/20 px-3">
        <div className="flex min-w-0 items-center gap-2" data-tauri-drag-region="false">
          <WindowControls />
          <span className="ml-1 truncate text-xs font-medium tracking-wide text-muted-foreground">
            Coda
          </span>
        </div>
        <nav className="flex items-center gap-1" data-tauri-drag-region="false">
          <Tab active={view === "meeting"} onClick={() => setView("meeting")}>
            Meeting
          </Tab>
          <Tab active={view === "agent"} onClick={() => setView("agent")}>
            Agent
          </Tab>
          <Tab active={view === "setup"} onClick={() => setView("setup")}>
            Setup
            {setup?.firstRun ? <span className="ml-1 size-1.5 rounded-full bg-stealth" /> : null}
          </Tab>
          <Tab active={view === "audit"} onClick={() => setView("audit")}>
            Audit
          </Tab>
        </nav>
        <div className="w-[76px]" />
      </DragRegion>
      <AppErrorBoundary>
        {view === "setup" ? (
          <SetupPanel />
        ) : view === "agent" ? (
          <AgentWorkspace />
        ) : view === "audit" ? (
          <AuditView />
        ) : (
          <Dashboard embedded />
        )}
        <ScreenAskOverlay />
      </AppErrorBoundary>
    </div>
  );
}

function Tab({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`inline-flex items-center rounded-full px-3 py-1 text-xs ${
        active ? "bg-white/10 text-foreground" : "text-muted-foreground hover:text-foreground"
      }`}
    >
      {children}
    </button>
  );
}
