export type IssueTab = "all" | "code" | "docs" | "design" | "testing" | "community";

export type IssueLike = {
  labels: { name: string }[];
};

export function categorizeIssue(issue: IssueLike): IssueTab[] {
  const names = issue.labels.map((label) => label.name.toLowerCase());
  const tabs: IssueTab[] = ["all"];
  const has = (pattern: RegExp) => names.some((name) => pattern.test(name));

  if (has(/doc|readme|guide|tutorial/)) tabs.push("docs");
  if (has(/design|ui|ux|icon/)) tabs.push("design");
  if (has(/test|qa|perf/)) tabs.push("testing");
  if (has(/community|question|discussion/)) tabs.push("community");
  if (has(/bug|enhancement|feature|code|good first|help wanted/) || tabs.length === 1) {
    tabs.push("code");
  }
  return tabs;
}
