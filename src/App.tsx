import { useEffect } from "react";

import { StealthPill } from "@/features/stealth/stealth-pill";
import { useCaptureStore } from "@/store/capture";
import { useCoachStore } from "@/store/coach";
import { useStealthStore } from "@/store/stealth";
import { Dashboard } from "@/views/dashboard";

export default function App() {
  const status = useStealthStore((state) => state.status);
  const refresh = useStealthStore((state) => state.refresh);
  const initCapture = useCaptureStore((state) => state.init);
  const initCoach = useCoachStore((state) => state.init);

  // The window may already be excluded from a previous session, so the first
  // thing the UI does is ask the OS what is actually true.
  useEffect(() => {
    void refresh();
    void initCapture();
    void initCoach();
  }, [refresh, initCapture, initCoach]);

  return status?.pillMode ? <StealthPill /> : <Dashboard />;
}
