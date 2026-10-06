# Sandbox

```
Agent → Sandbox Manager → isolated temp workspace → allowlisted process
```

Each run gets a `sandbox_id` and a directory under
`{app_data}/agent/sandboxes/{id}`. Resources are deleted when the step
finishes unless the user asked to keep them.

Limits: timeout, output size, working directory jail, no credentials dir,
network off by default.

Allowed programs: `python3`/`python`, `node`. `SHELL_EXECUTE` is a small
allowlist (`ls`, `pwd`, `date`, `echo` equivalents on the workspace) — not
`cmd.exe` / `bash -c` with arbitrary strings.

Computer-use automation is a separate policy surface and defaults to DENY.
