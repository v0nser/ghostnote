# Agents

Ghost Note supports multiple assistant profiles. Each profile is a local
configuration: prompt, model preference, tool allowlist, memory scope, and
approval policy. Profiles cannot escalate their own permissions.

## Built-in profiles

| Profile | Purpose | Tools | Notes |
| --- | --- | --- | --- |
| Meeting Assistant | Live interview / meeting copilot | memory, datetime, notification | Existing Whisper → Ollama path |
| Personal Assistant | General goals | search, fetch, files (workspace), memory, schedule | Email send always needs approval |
| Research Agent | Compare options, reports | search, fetch, memory, files write | No send / delete / shell |
| Coding Agent | Local code in the workspace | files, sandbox execute, search | Email send disabled |
| Job Search Agent | Research roles, draft applications | search, fetch, files, memory | Submit / send = approval |

## Task lifecycle

`IDLE → UNDERSTANDING → PLANNING → EXECUTING → REFLECTING → COMPLETED`

Side paths: `WAITING_FOR_APPROVAL`, `WAITING_FOR_TOOL`, `REPLANNING`,
`PAUSED`, `CANCELLED`, `FAILED`, `SCHEDULED`.

Controls: pause, resume, cancel, retry, replan, continue.

Tasks persist under `{app_data}/agent/tasks/` and survive restarts.
