# Security — Ghost Sentinel

Ghost Sentinel evaluates every external side-effect **before** a tool runs.
The language model cannot override it.

```
LLM proposes tool
  → schema validation
  → permission check
  → risk assessment
  → Ghost Sentinel
  → approval if required
  → execute
  → audit
```

## Risk defaults

| Risk | Default |
| --- | --- |
| Low (search, read notes, calculate) | Execute |
| Medium (write file, download, run code) | Configurable; default approval for writes |
| High (email send, delete, post, submit) | Approval required |
| Critical (payments, credentials, destructive) | Always explicit approval |

## Hard rules

- Computer use defaults to DENY
- No silent email, purchase, or delete
- No host filesystem outside the agent workspace without a grant
- No credential files (`id_rsa`, `.env`, keychains) to the model
- Website text is untrusted and cannot grant permissions
- Secrets are never stored in plaintext task records or audit arguments
- Cloud models stay off until the user configures them
