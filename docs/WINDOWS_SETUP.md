# Windows setup

Coda on Windows is a normal Tauri app plus two local extras: a Whisper
model and Ollama. You do **not** need to clone the repo to *use* a signed
installer. You do need the steps below to *build* or to finish first-run
setup.

Data lives in `%USERPROFILE%\.coda`. If you used Ghost Note before, Coda
copies that app data on first launch and does not delete it.

## Use the installer (when published)

1. Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if Windows 10 asks (Windows 11 already has it). The NSIS bundle can download it.
2. Run the Coda setup `.exe` (per-user, no admin).
3. Open Coda. The **Setup** tab downloads the speech model and checks Ollama.
4. Install [Ollama for Windows](https://ollama.com/download/windows) and pull a small model:

```
ollama pull llama3.2:3b
```

`llama3.2:3b` or `qwen2.5:3b` replies much faster than a 8B+ model on a laptop.

## Build from source

Requirements:

- Node.js 20+
- Rust stable ([rustup](https://rustup.rs))
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the “Desktop development with C++” workload
- CMake + Git (for the Whisper sidecar)
- Ollama

From PowerShell in the repo:

```powershell
npm install
.\scripts\setup-whisper.ps1
npm run tauri:dev
```

`setup-whisper.ps1` builds `whisper-cli` as a Tauri sidecar and downloads
`ggml-small.en.bin` into `%USERPROFILE%\.coda\models`.

If you skip the script, the in-app Setup tab can still download the model
once the sidecar exists.

## System audio

Windows uses WASAPI loopback. No extra driver. Grant microphone permission
when Windows asks. Participant audio is whatever the speakers are playing.

## Packaging

```
npm run tauri:build
npm run installers:publish
```

Must run on Windows to produce `Coda-Setup.exe`.
