# Connectors

Connectors are explicit, revocable integrations.

```
id · name · type · permissions · auth_status · capabilities
```

Do not implement Gmail, Calendar, Slack, or Notion in this phase.

**Reference connector:** Local Notes. It reads and writes markdown files
inside the agent workspace. No OAuth. No extra permissions.

Future connectors must request the minimum permission set (e.g. email
`DRAFT` before `SEND`) and never see the credentials vault as model context.
