# Ghost Note crash analysis

Diagnosis of why the desktop app appears to close, why voice commands fail, and why computer-use requests never change the machine. No Python/FastAPI process exists. There is no WebSocket layer. Control flow is Tauri IPC plus local events.

## Architecture (what actually runs)

| Process | Role |
| --- | --- |
| `ghostnote` (Tauri / Rust) | UI host, agent, audio, tools |
| React webview | Meeting + Agent UI |
| `whisper-cli` sidecar | Speech-to-text (one process per clip) |
| Ollama at `127.0.0.1:11434` | Optional LLM. Not required for simple “open X” commands after this change |

There is no separate frontend Node process in a packaged build. `npm run tauri:dev` runs Vite only as a page server.

---

## Crash 1 — macOS TCC abort (real process kill)

### Crash source

`Termination Reason: Namespace TCC, Code 0`

macOS kills the process when WebKit Speech Recognition (`SFSpeechRecognizer`) is used without `NSSpeechRecognitionUsageDescription` in the running binary’s Info.plist.

`tauri dev` launches a **bare** `target/debug/ghostnote` binary, not a `.app` bundle. `src-tauri/Info.plist` is merged only into packaged apps. The usage string therefore does not exist on the process TCC inspects.

### Reproduction

1. `npm run tauri:dev`
2. Open Agent
3. Press the microphone (older builds called `webkitSpeechRecognition`)
4. Process exits immediately; Console shows TCC / `NSSpeechRecognitionUsageDescription`

### Stack (abridged)

```
Thread crashed: Dispatch queue: com.apple.root.default-qos
__abort_with_payload
__TCC_CRASHING_DUE_TO_PRIVACY_VIOLATION__
__TCCAccessRequest_block_invoke
```

Triggered from a WebKit worker, not the Rust main thread.

### Affected component

Frontend voice fallback (`src/lib/voice.ts`, removed). WebView → Speech Recognition → TCC.

### Root cause

Browser Speech Recognition was used as a Whisper fallback. On an unbundled debug binary this is a fatal privacy violation, not a recoverable JS error.

### Fix

- Never construct `SpeechRecognition` / `webkitSpeechRecognition`.
- Voice uses the local Whisper sidecar and native microphone capture only.
- Packaged builds still ship `NSSpeechRecognitionUsageDescription` in `src-tauri/Info.plist` as defense in depth.

### Regression test

`transcribe::tests::microphone_clips_are_queued_for_voice_commands` plus the voice lifecycle tests in `agent::intent` / computer adapters. Manual: hold the mic on `tauri dev` — the process must stay alive.

---

## Crash 2 — window close looks like a quit

### Crash source

`src-tauri/src/lib.rs` `WindowEvent::CloseRequested` hides the window and prevents destroy. The custom chrome X calls `appWindow().close()`.

### Reproduction

Click the window close control.

### Stack

None. The process is still running.

### Affected component

Window lifecycle / custom chrome.

### Root cause

Users interpret hide-as-close as a crash. Combined with Crash 1, “it closed” had two different meanings.

### Fix

Keep hide-on-close (so a stray click does not kill work). Task state lives on `AgentRuntime`, not the React tree. Dock Reopen shows the window again.

### Regression test

Close the window during a task; reopen from the Dock. Task list must still be present.

---

## Crash 3 — meeting transcriber dropped microphone clips

### Crash source

Not a process kill. Voice “did nothing.”

`src-tauri/src/audio/pipeline.rs` discarded `Speaker::You` utterances (meeting design: transcribe the other person only). `src-tauri/src/transcribe/mod.rs` `enqueue()` also ignored non-participant clips and could shut the worker down when the first clip was You-only.

### Reproduction

Hold the mic, speak “open Safari”, release. Capture starts and stops; no transcript; UI says it heard nothing.

### Affected component

Audio pipeline + Whisper queue + `agent_finish_voice`.

### Root cause

Voice commands reuse the meeting capture path, which was intentionally not transcribing the user. Finish also aborted before Whisper flushed.

### Fix

`CaptureOptions.transcribe_microphone` for voice sessions. Transcriber queues You clips. `agent_finish_voice` waits for Whisper, then returns the final transcript only.

### Regression test

`transcribe::tests::microphone_clips_are_queued_for_voice_commands`.

---

## Failure 4 — “Open Apple Music” never touches the OS

### Crash source

None. The agent completed with a text answer.

### Reproduction

Type or speak “Open Apple Music.”

### Affected component

`planner.rs` + `runtime.rs` + `tools.rs`.

### Root cause

There was no `OPEN_APPLICATION` (or any computer-use) tool. The planner produced “Understand” + “Write the answer.” The LLM was asked to reply in text. `SHELL_EXECUTE` is disabled. `computer_use_enabled` defaulted to false. `BROWSER_OPEN` only fetched HTTP.

### Fix

`FastIntentRouter` for deterministic commands, plus `OPEN_APPLICATION` / `OPEN_URL` / `OPEN_FOLDER` / `OPEN_FILE` implemented by platform adapters that launch and **verify**. The model is not asked to invent `open -a` strings for these.

### Regression test

`agent::intent` and `agent::computer` unit tests (resolver aliases, Google/YouTube URLs, open-application routing). Failed tools must mark the task `FAILED` and leave the process running.

---

## Other findings (non-fatal)

| Item | Notes |
| --- | --- |
| `unwrap` / `expect` | Startup only: Tauri builder, reqwest client, audio thread spawn. Runtime tool/agent paths return `Result`. |
| `process::exit` / `app.exit` | Not used. |
| Python / FastAPI | Not in this app. |
| WebSockets | Not used. |
| Stealth AppKit | Must stay on the main thread (`stealth/mod.rs`). Off-main use would abort. |
| Unknown tools | `run_named_tool` used to return `Ok(())` and look like success. Now a structured error. |
| Frontend | No error boundary existed. Added. Unhandled promise rejections must not unmount the shell. |

## Global rule

A failed model, microphone, tool, or missing app transitions the **task** to `FAILED` or `WAITING_FOR_USER`. It must not exit the Tauri process.
