"use client";

import { useEffect, useState } from "react";
import { Github } from "lucide-react";

import { repoUrl } from "@/lib/repo";

export function GitHubBadge() {
  const [stars, setStars] = useState<number | null>(null);
  const [contributors, setContributors] = useState<number | null>(null);

  useEffect(() => {
    fetch("/api/github", { cache: "no-store" })
      .then((res) => res.json())
      .then((data) => {
        if (typeof data?.stats?.stars === "number") setStars(data.stats.stars);
        if (typeof data?.stats?.contributors === "number") setContributors(data.stats.contributors);
      })
      .catch(() => undefined);
  }, []);

  return (
    <div className="flex flex-wrap items-center gap-3">
      {contributors !== null ? (
        <p className="text-xs text-white/35">
          © {new Date().getFullYear()} GhostNote
          {contributors > 0 ? ` · ${contributors} contributor${contributors === 1 ? "" : "s"}` : ""}
        </p>
      ) : (
        <p className="text-xs text-white/35">© {new Date().getFullYear()} GhostNote</p>
      )}
      <a
        href={repoUrl()}
        aria-label="GitHub"
        className="focus-ring inline-flex items-center gap-2 rounded-full border border-white/10 px-3 py-1.5 text-xs text-mist transition hover:border-cyan/40 hover:text-white"
      >
        <Github className="size-3.5" />
        {stars === null ? "GitHub" : `${stars.toLocaleString()} stars`}
      </a>
    </div>
  );
}
