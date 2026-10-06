# Builds the whisper.cpp sidecar and installs the English speech model on Windows.
#
# Run from the repo root in PowerShell:
#   .\scripts\setup-whisper.ps1 [model]
#
# `model` defaults to small.en for accent accuracy

param(
  [string]$Model = "small.en"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Tooling = Join-Path $RepoRoot ".tooling"
$Binaries = Join-Path $RepoRoot "src-tauri\binaries"
$ModelDir = if ($env:CODA_HOME) {
  Join-Path $env:CODA_HOME "models"
} else {
  Join-Path $env:USERPROFILE ".coda\models"
}

foreach ($tool in @("cmake", "git", "rustc")) {
  if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
    throw "missing required tool: $tool"
  }
}

$HostInfo = rustc -vV | Select-String "^host:"
$Triple = ($HostInfo -split "\s+")[1]
if (-not $Triple) { throw "could not read rustc host triple" }

Write-Host "==> Fetching whisper.cpp"
New-Item -ItemType Directory -Force -Path $Tooling | Out-Null
$Whisper = Join-Path $Tooling "whisper.cpp"
if (Test-Path (Join-Path $Whisper ".git")) {
  git -C $Whisper pull --ff-only
} else {
  git clone --depth 1 https://github.com/ggml-org/whisper.cpp.git $Whisper
}

Write-Host "==> Building whisper-cli (static)"
$Build = Join-Path $Whisper "build-static"
cmake -S $Whisper -B $Build `
  -DCMAKE_BUILD_TYPE=Release `
  -DBUILD_SHARED_LIBS=OFF `
  -DWHISPER_BUILD_TESTS=OFF `
  -DWHISPER_BUILD_SERVER=OFF
cmake --build $Build --config Release --target whisper-cli

$Built = @(
  (Join-Path $Build "bin\Release\whisper-cli.exe"),
  (Join-Path $Build "Release\whisper-cli.exe"),
  (Join-Path $Build "bin\whisper-cli.exe")
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $Built) { throw "whisper-cli.exe was not produced" }

New-Item -ItemType Directory -Force -Path $Binaries | Out-Null
$Sidecar = Join-Path $Binaries "whisper-cli-$Triple.exe"
Copy-Item $Built $Sidecar -Force

Write-Host "==> Installing model ggml-$Model.bin"
New-Item -ItemType Directory -Force -Path $ModelDir | Out-Null
$ModelPath = Join-Path $ModelDir "ggml-$Model.bin"
if (-not (Test-Path $ModelPath)) {
  $Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-$Model.bin"
  Invoke-WebRequest -Uri $Url -OutFile $ModelPath
}

Write-Host ""
Write-Host "Done."
Write-Host "  sidecar: $Sidecar"
Write-Host "  model:   $ModelPath"
Write-Host "Then: npm install && npm run tauri:dev"
