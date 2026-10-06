# Browser agent

```
Agent → Browser tool API → isolated context → page
```

Current release:

- `WEB_SEARCH` and `WEB_FETCH` run in-process with SSRF protections
- `BROWSER_OPEN` fetches and extracts readable text
- Click / type / screenshot are registered and policy-gated; a Playwright
  sidecar lands in Phase 6

Every browser action emits `browser.action.started|completed|failed` with
`task_id`, `step_id`, `action`, `url`, timestamp, duration, and a short
result summary.

The model never launches arbitrary OS browsers or attaches to the user’s
logged-in daily driver.
