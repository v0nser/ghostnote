# Threat model

## Assets

- Meeting audio and transcripts
- Agent task history and artifacts
- User memory and preferences
- Workspace files
- Future connector tokens (never plaintext in the main DB)
- Local model prompts (may include private context)

## Attack surfaces

- Tool arguments produced by the LLM
- Web page content (indirect prompt injection)
- File names and downloaded bytes
- Scheduler callbacks
- Website / checkout (separate process; no desktop secrets)
- Sidecar processes (Whisper)

## Trust boundaries

1. React UI is untrusted for policy. It can only invoke commands.
2. Model output is untrusted data.
3. Web content is untrusted data.
4. Ghost Sentinel + path/SSRF/schema checks are the security boundary.
5. Ollama is trusted as a local process the user installed.

## Assumptions

- The user controls their machine.
- Ollama is bound to localhost.
- The app data directory is not world-writable.
- No multi-tenant server.

## Mitigations

- Deterministic Sentinel
- Workspace jail + path canonicalization
- SSRF denylist
- Approval for high/critical risk
- Sanitized audit log
- Untrusted-content wrapper around fetched HTML
- Computer-use default DENY

## Known limitations

- A local attacker with the same OS user can read app data
- Playwright isolation is not shipped yet
- Sandbox is a process jail, not a VM
- Prompt injection can waste time / produce bad plans; it must not gain
  permissions or read secrets
