"use client";

import { type FormEvent, useState } from "react";

import { MagneticButton } from "@/components/magnetic-button";
import { newIssueUrl } from "@/lib/repo";

const LABELS = [
  { id: "bug", label: "Bug" },
  { id: "enhancement", label: "Feature" },
  { id: "documentation", label: "Docs" },
  { id: "design", label: "Design" },
  { id: "testing", label: "Testing" },
  { id: "question", label: "Question" },
];

export function ReportIssue() {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [label, setLabel] = useState("bug");

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!title.trim()) return;
    window.open(
      newIssueUrl({
        title: title.trim(),
        body: body.trim() || "Filed from the GhostNote landing page while testing.",
        labels: label,
      }),
      "_blank",
      "noreferrer",
    );
  };

  return (
    <form onSubmit={submit} className="rounded-3xl border border-white/10 bg-black/40 p-5">
      <h3 className="text-lg font-semibold">Found something? File it here.</h3>
      <p className="mt-1 text-sm text-mist">
        Opens a GitHub issue on the live repo. After you submit there, it appears in this list.
      </p>
      <label className="mt-4 block text-xs text-white/70" htmlFor="issue-title">
        Title
      </label>
      <input
        id="issue-title"
        required
        value={title}
        onChange={(event) => setTitle(event.target.value)}
        placeholder="Stealth toggle flickers on macOS…"
        className="focus-ring mt-1 w-full rounded-full border border-white/15 bg-white/5 px-4 py-2.5 text-sm"
      />
      <label className="mt-3 block text-xs text-white/70" htmlFor="issue-body">
        What happened
      </label>
      <textarea
        id="issue-body"
        value={body}
        onChange={(event) => setBody(event.target.value)}
        rows={3}
        placeholder="Steps to reproduce, expected vs actual."
        className="focus-ring mt-1 w-full rounded-2xl border border-white/15 bg-white/5 px-4 py-2.5 text-sm"
      />
      <div className="mt-3 flex flex-wrap gap-2" role="group" aria-label="Issue type">
        {LABELS.map((item) => (
          <button
            key={item.id}
            type="button"
            onClick={() => setLabel(item.id)}
            className={`focus-ring rounded-full border px-3 py-1 text-xs ${
              label === item.id ? "border-accent-cyan/50 bg-white/10 text-white" : "border-white/10 text-mist"
            }`}
          >
            {item.label}
          </button>
        ))}
      </div>
      <MagneticButton type="submit" className="mt-4">
        Open GitHub issue
      </MagneticButton>
    </form>
  );
}
