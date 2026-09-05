export function getRepoSlug() {
  const raw = process.env.NEXT_PUBLIC_GITHUB_REPO || process.env.GITHUB_REPO || "v0nser/ghostnote";
  return raw.replace(/^https?:\/\/github\.com\//, "").replace(/\.git$/, "").replace(/\/$/, "");
}

export function repoUrl(path = "") {
  const slug = getRepoSlug();
  return `https://github.com/${slug}${path}`;
}

export function newIssueUrl(input?: { title?: string; body?: string; labels?: string }) {
  const params = new URLSearchParams();
  if (input?.title) params.set("title", input.title);
  if (input?.body) params.set("body", input.body);
  if (input?.labels) params.set("labels", input.labels);
  const query = params.toString();
  return repoUrl(`/issues/new${query ? `?${query}` : ""}`);
}
