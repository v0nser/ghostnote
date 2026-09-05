import { categorizeIssue, type IssueTab } from "@/lib/github-issues";
import { getRepoSlug, repoUrl } from "@/lib/repo";

export type { IssueTab };

export type GitHubIssue = {
  id: number;
  number: number;
  title: string;
  html_url: string;
  labels: { name: string; color: string }[];
  user?: string;
  createdAt?: string;
  comments?: number;
};

export type GitHubContributor = {
  login: string;
  avatar_url: string;
  html_url: string;
  contributions: number;
};

export type GitHubStats = {
  stars: number;
  forks: number;
  contributors: number;
  commits: number;
  openIssues: number;
  closedPrs: number;
};

export type RecentMerge = {
  title: string;
  user: string;
  url: string;
  mergedAt: string;
};

type RawIssue = {
  id: number;
  number: number;
  title: string;
  html_url: string;
  labels?: { name: string; color: string }[];
  user?: { login: string };
  created_at?: string;
  comments?: number;
  pull_request?: unknown;
};

async function github<T>(path: string): Promise<{ data: T | null; status: number }> {
  const slug = getRepoSlug();
  try {
    const response = await fetch(`https://api.github.com/repos/${slug}${path}`, {
      headers: {
        Accept: "application/vnd.github+json",
        "User-Agent": "ghostnote-landing",
        ...(process.env.GITHUB_TOKEN ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` } : {}),
      },
      cache: "no-store",
    });
    if (!response.ok) return { data: null, status: response.status };
    return { data: (await response.json()) as T, status: response.status };
  } catch {
    return { data: null, status: 0 };
  }
}

function toIssue(item: RawIssue): GitHubIssue {
  return {
    id: item.id,
    number: item.number,
    title: item.title,
    html_url: item.html_url,
    labels: (item.labels ?? []).map((label) => ({ name: label.name, color: label.color })),
    user: item.user?.login,
    createdAt: item.created_at,
    comments: item.comments,
  };
}

export async function getGitHubBundle() {
  const slug = getRepoSlug();
  const [repo, contributors, issuesRes, pulls] = await Promise.all([
    github<{
      stargazers_count: number;
      forks_count: number;
      open_issues_count: number;
      html_url: string;
    }>(""),
    github<GitHubContributor[]>("/contributors?per_page=10"),
    github<RawIssue[]>("/issues?state=open&per_page=50"),
    github<{ title: string; user: { login: string }; html_url: string; merged_at: string | null }[]>(
      "/pulls?state=closed&per_page=8",
    ),
  ]);

  const openIssues = (issuesRes.data ?? []).filter((item) => !item.pull_request).map(toIssue);
  const grouped: Record<IssueTab, GitHubIssue[]> = {
    all: openIssues,
    code: [],
    docs: [],
    design: [],
    testing: [],
    community: [],
  };
  for (const issue of openIssues) {
    for (const tab of categorizeIssue(issue)) {
      if (tab !== "all") grouped[tab].push(issue);
    }
  }

  const commitCount = contributors.data?.reduce((sum, person) => sum + (person.contributions ?? 0), 0) ?? 0;
  const closedPrs = pulls.data?.length ?? 0;

  return {
    repo: repoUrl(),
    slug,
    live: Boolean(repo.data),
    status: repo.status,
    error:
      repo.status === 404
        ? "not_found"
        : repo.status === 403
          ? "rate_limited"
          : repo.data
            ? null
            : "unavailable",
    stats: {
      stars: repo.data?.stargazers_count ?? 0,
      forks: repo.data?.forks_count ?? 0,
      contributors: contributors.data?.length ?? 0,
      commits: commitCount,
      openIssues: openIssues.length,
      closedPrs,
    } satisfies GitHubStats,
    contributors: contributors.data ?? [],
    issues: grouped,
    merges:
      pulls.data
        ?.filter((item) => item.merged_at)
        .slice(0, 5)
        .map((item) => ({
          title: item.title,
          user: item.user.login,
          url: item.html_url,
          mergedAt: item.merged_at as string,
        })) ?? [],
    newIssueUrl: repoUrl("/issues/new"),
    issuesUrl: repoUrl("/issues"),
  };
}
