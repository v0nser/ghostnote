"use client";

import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion, useInView } from "framer-motion";
import { useRef } from "react";
import { Briefcase, Code2, Github, Rocket } from "lucide-react";

import { MagneticButton } from "@/components/magnetic-button";
import { ReportIssue } from "@/components/report-issue";
import { TiltCard } from "@/components/tilt-card";
import { categorizeIssue, type IssueTab } from "@/lib/github-issues";
import { getRepoSlug, repoUrl } from "@/lib/repo";
import { cn } from "@/lib/utils";

type Issue = {
  id: number;
  number: number;
  title: string;
  html_url: string;
  labels: { name: string; color: string }[];
  user?: string;
};
type Contributor = { login: string; avatar_url: string; html_url: string; contributions: number };
type Merge = { title: string; user: string; url: string; mergedAt: string };
type TabId = IssueTab;

const TABS: { id: TabId; label: string; copy: string }[] = [
  { id: "all", label: "All issues", copy: "Live open issues from the GitHub repo. File one and it shows up here." },
  { id: "code", label: "Code", copy: "Bugs, features, and unlabeled work items." },
  { id: "docs", label: "Docs", copy: "Documentation, guides, and translations." },
  { id: "design", label: "Design", copy: "UI, icons, and visual polish." },
  { id: "testing", label: "Testing", copy: "Tests, QA, and performance." },
  { id: "community", label: "Community", copy: "Questions and discussion." },
];

function useCountUp(target: number, active: boolean) {
  const [value, setValue] = useState(0);
  useEffect(() => {
    if (!active) return;
    const start = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const t = Math.min(1, (now - start) / 900);
      setValue(Math.round(target * t));
      if (t < 1) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [active, target]);
  return value;
}

export function Contribute() {
  const [tab, setTab] = useState<TabId>("all");
  const [updatedAt, setUpdatedAt] = useState<string>("");
  const [data, setData] = useState<{
    repo: string;
    slug: string;
    live: boolean;
    status: number;
    error?: string | null;
    stats: { stars: number; forks: number; contributors: number; commits: number; openIssues: number; closedPrs: number };
    contributors: Contributor[];
    issues: Record<string, Issue[]>;
    merges: Merge[];
    newIssueUrl: string;
    issuesUrl: string;
  } | null>(null);
  const statsRef = useRef<HTMLDivElement>(null);
  const statsInView = useInView(statsRef, { once: true });

  const load = useCallback(() => {
    const slug = getRepoSlug();
    Promise.all([
      fetch("/api/github", { cache: "no-store" }).then((res) => res.json()),
      fetch(`https://api.github.com/repos/${slug}/issues?state=open&per_page=50`, {
        headers: { Accept: "application/vnd.github+json" },
      }).then((res) => (res.ok ? res.json() : null)),
    ])
      .then(([payload, directIssues]) => {
        if (Array.isArray(directIssues)) {
          const open = (directIssues as { id: number; number: number; title: string; html_url: string; labels?: { name: string; color: string }[]; user?: { login: string }; pull_request?: unknown }[])
            .filter((item) => !item.pull_request)
            .map((item) => ({
              id: item.id,
              number: item.number,
              title: item.title,
              html_url: item.html_url,
              labels: item.labels ?? [],
              user: item.user?.login,
            }));
          const grouped: Record<TabId, Issue[]> = {
            all: open,
            code: [],
            docs: [],
            design: [],
            testing: [],
            community: [],
          };
          for (const issue of open) {
            for (const bucket of categorizeIssue(issue)) {
              if (bucket !== "all") grouped[bucket].push(issue);
            }
          }
          payload.issues = grouped;
          payload.live = true;
          payload.stats = {
            ...payload.stats,
            openIssues: open.length,
          };
        }
        payload.slug = slug;
        payload.repo = repoUrl();
        payload.newIssueUrl = repoUrl("/issues/new");
        payload.issuesUrl = repoUrl("/issues");
        setData(payload);
        setUpdatedAt(new Date().toLocaleTimeString());
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    load();
    const id = window.setInterval(load, 20000);
    const onFocus = () => load();
    window.addEventListener("focus", onFocus);
    return () => {
      window.clearInterval(id);
      window.removeEventListener("focus", onFocus);
    };
  }, [load]);

  const issues = data?.issues[tab] ?? [];
  const stats = data?.stats ?? { stars: 0, forks: 0, contributors: 0, commits: 0, openIssues: 0, closedPrs: 0 };
  const stars = useCountUp(stats.stars, statsInView);
  const forks = useCountUp(stats.forks, statsInView);
  const contributorCount = useCountUp(stats.contributors, statsInView);
  const repo = repoUrl();
  const slug = getRepoSlug();

  return (
    <section id="contribute" className="relative overflow-hidden px-5 py-24">
      <div className="pointer-events-none absolute inset-0 bg-[linear-gradient(180deg,#000_0%,#121214_55%,#000_100%)]" />
      <div className="pointer-events-none absolute inset-0 opacity-[0.08] [background-image:linear-gradient(rgba(255,255,255,0.12)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.12)_1px,transparent_1px)] [background-size:48px_48px]" />

      <div className="relative mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-5xl">Build the Future of Stealth AI With Us</h2>
        <p className="mx-auto mt-4 max-w-2xl text-center text-lg text-white/80 md:text-2xl">
          GhostNote is open source. Issues below are live from GitHub — test the product, file what you find.
        </p>
        <p className="mt-3 text-center text-sm text-mist">
          Connected to{" "}
          <a href={repo} className="link-underline text-accent-cyan" target="_blank" rel="noreferrer">
            {slug}
          </a>
          {updatedAt ? ` · refreshed ${updatedAt}` : ""}
        </p>

        <div className="mt-12 grid gap-5 md:grid-cols-3">
          <TiltCard>
            <Code2 className="size-6 text-accent-cyan" />
            <h3 className="mt-4 text-lg font-semibold">Cutting-Edge Tech Stack</h3>
            <p className="mt-2 text-sm text-mist">Work with Rust, Tauri, WebRTC, local AI, and more</p>
            <div className="mt-4 flex flex-wrap gap-2">
              {["Rust", "TypeScript", "WebAssembly", "Ollama"].map((badge) => (
                <span key={badge} className="rounded-full border border-white/10 px-2.5 py-1 text-[11px] text-white/80">
                  {badge}
                </span>
              ))}
            </div>
          </TiltCard>
          <TiltCard>
            <Rocket className="size-6 text-accent-cyan" />
            <h3 className="mt-4 text-lg font-semibold">Real Impact</h3>
            <p className="mt-2 text-sm text-mist">Ship code that testers actually run. Pick a live issue and open a PR.</p>
            <p className="mt-4 text-sm text-white/80">
              {stats.openIssues} open issues · {stats.contributors} contributors · {stats.stars} stars
            </p>
          </TiltCard>
          <TiltCard>
            <Briefcase className="size-6 text-accent-cyan" />
            <h3 className="mt-4 text-lg font-semibold">Start With an Issue</h3>
            <p className="mt-2 text-sm text-mist">No placeholder tickets. If the list is empty, be the first to file one.</p>
            <MagneticButton href={data?.newIssueUrl ?? repoUrl("/issues/new")} className="mt-4">
              File an issue
            </MagneticButton>
          </TiltCard>
        </div>

        <div className="mt-16 grid gap-6 lg:grid-cols-[1.2fr_0.8fr]">
          <div>
            <div className="flex gap-2 overflow-x-auto pb-2" role="tablist" aria-label="Live GitHub issues">
              {TABS.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  role="tab"
                  aria-selected={tab === item.id}
                  className={cn(
                    "focus-ring whitespace-nowrap rounded-full border px-4 py-2 text-sm",
                    tab === item.id ? "border-accent-cyan/50 bg-white/10 text-white" : "border-white/10 text-mist",
                  )}
                  onClick={() => setTab(item.id)}
                >
                  {item.label}
                  {data ? ` (${data.issues[item.id]?.length ?? 0})` : ""}
                </button>
              ))}
            </div>
            <AnimatePresence mode="wait">
              <motion.div
                key={tab}
                role="tabpanel"
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                className="mt-6 rounded-3xl border border-white/10 bg-charcoal p-6"
              >
                <p className="text-sm text-mist">{TABS.find((item) => item.id === tab)?.copy}</p>
                {!data?.live ? (
                  <p className="mt-5 text-sm text-rose-300">
                    {data?.error === "rate_limited"
                      ? "GitHub rate-limited this request. Add a GITHUB_TOKEN in website/.env.local, or wait a minute and refresh."
                      : data?.error === "not_found"
                        ? `Repository ${slug} was not found. Confirm it is public at ${repo}.`
                        : `Could not load issues from ${slug}. If GitHub rate-limited the request, add a GITHUB_TOKEN and refresh.`}
                  </p>
                ) : issues.length === 0 ? (
                  <p className="mt-5 text-sm text-white/70">
                    No open {tab === "all" ? "" : `${tab} `}issues yet. File one on GitHub and it will show up here within
                    20 seconds.
                  </p>
                ) : (
                  <ul className="mt-5 space-y-3">
                    {issues.map((issue, index) => (
                      <motion.li
                        key={issue.id}
                        initial={{ opacity: 0, x: -12 }}
                        animate={{ opacity: 1, x: 0 }}
                        transition={{ delay: index * 0.04 }}
                        className="flex flex-col gap-3 rounded-2xl border border-white/8 bg-black/30 p-4 sm:flex-row sm:items-center sm:justify-between"
                      >
                        <div>
                          <a
                            href={issue.html_url}
                            className="link-underline text-sm font-medium text-white"
                            target="_blank"
                            rel="noreferrer"
                          >
                            #{issue.number} {issue.title}
                          </a>
                          <div className="mt-2 flex flex-wrap gap-2">
                            {issue.user ? <span className="text-xs text-mist">@{issue.user}</span> : null}
                            {issue.labels.map((label) => (
                              <span
                                key={label.name}
                                className="rounded-full px-2 py-0.5 text-[10px] uppercase tracking-wide text-ink"
                                style={{ background: `#${label.color}` }}
                              >
                                {label.name}
                              </span>
                            ))}
                          </div>
                        </div>
                        <MagneticButton href={issue.html_url} variant="secondary" className="shrink-0">
                          Open on GitHub
                        </MagneticButton>
                      </motion.li>
                    ))}
                  </ul>
                )}
                <div className="mt-6 flex flex-wrap gap-3">
                  <MagneticButton href={data?.issuesUrl ?? repoUrl("/issues")}>Browse all issues</MagneticButton>
                  <MagneticButton href={data?.newIssueUrl ?? repoUrl("/issues/new")} variant="secondary">
                    New issue
                  </MagneticButton>
                </div>
              </motion.div>
            </AnimatePresence>
          </div>
          <ReportIssue />
        </div>

        <div className="mt-16 grid gap-4 md:grid-cols-5">
          {[
            ["Fork the repository", `git clone ${repo}.git`, ["View on GitHub", repo]],
            ["Set up locally", "npm install\nnpm run dev", ["Read README", `${repo}#readme`]],
            ["Pick a live issue", "Start with an open issue from the list", ["Browse issues", `${repo}/issues`]],
            ["Open a pull request", "Push a branch and open a PR against main", ["Open a PR", `${repo}/compare`]],
            ["See it land", "Merged work shows up in Recent merges below", ["Contributors", "#contribute-leaderboard"]],
          ].map(([title, code, cta], index) => (
            <article key={title as string} className="rounded-3xl border border-white/10 bg-black/40 p-4">
              <p className="text-xs uppercase tracking-[0.16em] text-accent-cyan">Step {index + 1}</p>
              <h3 className="mt-2 text-sm font-semibold">{title as string}</h3>
              <pre className="mt-3 overflow-x-auto rounded-xl bg-black/60 p-3 font-mono text-[11px] text-white/70">
                {code as string}
              </pre>
              <a href={(cta as string[])[1]} className="link-underline mt-3 inline-block text-xs text-accent-cyan">
                {(cta as string[])[0]}
              </a>
            </article>
          ))}
        </div>

        <div id="contribute-leaderboard" className="mt-16 grid gap-8 lg:grid-cols-2">
          <div>
            <h3 className="text-2xl font-semibold">Contributors</h3>
            {(data?.contributors ?? []).length === 0 ? (
              <p className="mt-5 text-sm text-mist">No contributors listed yet. Merge the first PR to appear here.</p>
            ) : (
              <ol className="mt-5 space-y-3">
                {(data?.contributors ?? []).slice(0, 10).map((person, index) => (
                  <li
                    key={person.login}
                    className="flex items-center justify-between rounded-2xl border border-white/8 bg-charcoal px-4 py-3"
                  >
                    <div className="flex items-center gap-3">
                      <span className="w-6 text-sm text-mist">{index + 1}</span>
                      <a href={person.html_url} className="text-sm text-white hover:text-accent-cyan">
                        @{person.login}
                      </a>
                    </div>
                    <span className="text-xs text-mist">{person.contributions} commits</span>
                  </li>
                ))}
              </ol>
            )}
          </div>
          <div>
            <h3 className="text-2xl font-semibold">Recent merges</h3>
            {(data?.merges ?? []).length === 0 ? (
              <p className="mt-5 text-sm text-mist">No merged pull requests yet.</p>
            ) : (
              <ul className="mt-5 space-y-3">
                {(data?.merges ?? []).map((merge) => (
                  <li key={`${merge.url}-${merge.mergedAt}`} className="rounded-2xl border border-white/8 bg-charcoal px-4 py-3 text-sm">
                    <a href={merge.url} className="text-white hover:text-accent-cyan">
                      @{merge.user} merged “{merge.title}”
                    </a>
                    <p className="mt-1 text-xs text-mist">{timeAgo(merge.mergedAt)}</p>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        <div ref={statsRef} className="mt-16 grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-6">
          {[
            ["Stars", stars],
            ["Forks", forks],
            ["Contributors", contributorCount],
            ["Commits", stats.commits],
            ["Open issues", stats.openIssues],
            ["Closed PRs", stats.closedPrs],
          ].map(([label, value]) => (
            <div key={label as string} className="rounded-3xl border border-white/10 bg-black/40 p-4 text-center">
              <p className="font-mono text-2xl text-white">{Number(value).toLocaleString()}</p>
              <p className="mt-1 text-[11px] uppercase tracking-wide text-mist">{label as string}</p>
            </div>
          ))}
        </div>

        <div className="mt-16 rounded-[32px] border border-white/10 bg-black/50 p-8 text-center md:p-12">
          <h3 className="text-3xl font-semibold md:text-4xl">Ready to make an impact?</h3>
          <p className="mt-3 text-mist">Clone the repo, file an issue, or open a pull request.</p>
          <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
            <MagneticButton href={repo}>
              <span className="inline-flex items-center gap-2">
                <Github className="size-4" /> View repository
              </span>
            </MagneticButton>
            <MagneticButton href={data?.newIssueUrl ?? repoUrl("/issues/new")} variant="secondary">
              File an issue
            </MagneticButton>
          </div>
        </div>
      </div>
    </section>
  );
}

function timeAgo(value: string) {
  const delta = Date.now() - new Date(value).getTime();
  const hours = Math.max(1, Math.round(delta / 3600_000));
  if (hours < 24) return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  const days = Math.round(hours / 24);
  return `${days} day${days === 1 ? "" : "s"} ago`;
}
