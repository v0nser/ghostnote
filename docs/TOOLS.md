# Tools

Every tool implements `validate`, `execute`, `cancel`, `describe`, and `risk`.
The registry discovers builtins at startup. The model never calls a tool
directly — the orchestrator does, after Sentinel.

## Contract

```
name
description
input_schema
output_schema
risk_level
permissions_required
requires_confirmation
timeout
supports_cancellation
```

## Initial tools

| Name | Risk | Notes |
| --- | --- | --- |
| WEB_SEARCH | Low | DuckDuckGo HTML, no API key |
| WEB_FETCH | Low | http/https, SSRF blocked |
| BROWSER_OPEN | Medium | Controlled fetch + extract |
| BROWSER_CLICK / TYPE / SCROLL / SCREENSHOT | Medium | Registered; full Playwright is Phase 6 |
| FILES_LIST / READ / SEARCH | Low–Medium | Workspace jail |
| FILES_WRITE | Medium | Workspace only |
| FILES_DELETE | High | Always approval |
| CODE_EXECUTE / PYTHON_EXECUTE | Medium | Sandbox allowlist |
| SHELL_EXECUTE | High | Allowlisted binaries only, never a raw shell |
| CALCULATOR | Low | |
| DATETIME | Low | |
| MEMORY_SEARCH / WRITE / DELETE | Low–Medium | |
| NOTIFICATION | Low | Local UI event |
| TASK_SCHEDULE / TASK_CANCEL | Medium | |

Unrestricted host shell is not exposed.
