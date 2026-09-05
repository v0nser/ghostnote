import React from "react";
import ReactDOM from "react-dom/client";

import App from "@/App";
import "@/styles/globals.css";

// The window is undecorated, so the webview's own context menu and drag-and-
// drop affordances are the only ways stray OS chrome could appear over a
// shared screen. Both are disabled outright.
document.addEventListener("contextmenu", (event) => event.preventDefault());
document.addEventListener("dragover", (event) => event.preventDefault());
document.addEventListener("drop", (event) => event.preventDefault());

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
