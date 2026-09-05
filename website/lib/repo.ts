export const GITHUB_REPO_SLUG = "v0nser/ghostnote";

function normalizeRepoSlug(value: string) {
  return value.replace(/^https?:\/\/github\.com\//, "").replace(/\.git$/, "").replace(/\/$/, "");
}

export function getRepoSlug() {
  const configured = normalizeRepoSlug(
    process.env.NEXT_PUBLIC_GITHUB_REPO || process.env.GITHUB_REPO || GITHUB_REPO_SLUG,
  );
  if (!configured || configured === "ghostnote/ghostnote") return GITHUB_REPO_SLUG;
  return configured;
}

export function repoUrl(path = "") {
  return `https://github.com/${getRepoSlug()}${path}`;
}

export function newIssueUrl(input?: { title?: string; body?: string; labels?: string }) {
  const params = new URLSearchParams();
  if (input?.title) params.set("title", input.title);
  if (input?.body) params.set("body", input.body);
  if (input?.labels) params.set("labels", input.labels);
  const query = params.toString();
  return repoUrl(`/issues/new${query ? `?${query}` : ""}`);
}
