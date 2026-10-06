import React from "react";
import ReactDOM from "react-dom/client";

import App from "@/App";
import { AppErrorBoundary } from "@/components/error-boundary";
import { useAgentStore } from "@/store/agent";
import "@/styles/globals.css";

// The window is undecorated, so the webview's own context menu and drag-and-
// drop affordances are the only ways stray OS chrome could appear over a
// shared screen. Both are disabled outright.
document.addEventListener("contextmenu", (event) => event.preventDefault());
document.addEventListener("dragover", (event) => event.preventDefault());
document.addEventListener("drop", (event) => event.preventDefault());

window.addEventListener("error", (event) => {
  event.preventDefault();
  useAgentStore.setState({
    error: event.message || "Something went wrong. Coda is still running.",
  });
});

window.addEventListener("unhandledrejection", (event) => {
  event.preventDefault();
  const reason = event.reason instanceof Error ? event.reason.message : String(event.reason ?? "");
  useAgentStore.setState({
    error: reason || "A background task failed. Coda is still running.",
  });
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <AppErrorBoundary>
      <App />
    </AppErrorBoundary>
  </React.StrictMode>,
);
