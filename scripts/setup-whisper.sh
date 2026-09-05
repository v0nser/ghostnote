#!/usr/bin/env bash
#
# Builds the whisper.cpp sidecar and installs the speech model.
#
# Neither artefact is committed: the binary is platform-specific and the model
# is 141 MB. Run this once after cloning.
#
#   ./scripts/setup-whisper.sh [model]
#
# `model` defaults to base.en. Use small.en for better accuracy at ~3.5x the
# size and runtime.

set -euo pipefail

MODEL="${1:-base.en}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLING="$REPO_ROOT/.tooling"
BINARIES="$REPO_ROOT/src-tauri/binaries"

# Tauri resolves sidecars by target triple, so the binary must be named for the
# platform it was built on.
TRIPLE="$(rustc -vV | awk '/^host:/ {print $2}')"

case "$(uname -s)" in
  Darwin) MODEL_DIR="$HOME/Library/Application Support/com.ghostnote.app/models" ;;
  Linux)  MODEL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/com.ghostnote.app/models" ;;
  *)      echo "unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac

for tool in cmake git curl rustc; do
  command -v "$tool" >/dev/null || { echo "missing required tool: $tool" >&2; exit 1; }
done

echo "==> Fetching whisper.cpp"
mkdir -p "$TOOLING"
if [ -d "$TOOLING/whisper.cpp/.git" ]; then
  git -C "$TOOLING/whisper.cpp" pull --ff-only
else
  git clone --depth 1 https://github.com/ggml-org/whisper.cpp.git "$TOOLING/whisper.cpp"
fi

# BUILD_SHARED_LIBS=OFF is what makes this usable as a Tauri sidecar: the
# default build links six ggml dylibs via @rpath, none of which Tauri would
# bundle alongside the binary.
#
# GGML_METAL_EMBED_LIBRARY bakes the Metal shaders into the executable so it
# does not need to locate a .metallib at runtime.
echo "==> Building whisper-cli (static)"
cmake -S "$TOOLING/whisper.cpp" -B "$TOOLING/whisper.cpp/build-static" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DGGML_METAL=ON \
  -DGGML_METAL_EMBED_LIBRARY=ON \
  -DWHISPER_BUILD_TESTS=OFF \
  -DWHISPER_BUILD_SERVER=OFF
cmake --build "$TOOLING/whisper.cpp/build-static" --config Release -j"$(getconf _NPROCESSORS_ONLN)" --target whisper-cli

echo "==> Installing sidecar as whisper-cli-$TRIPLE"
mkdir -p "$BINARIES"
cp "$TOOLING/whisper.cpp/build-static/bin/whisper-cli" "$BINARIES/whisper-cli-$TRIPLE"
chmod +x "$BINARIES/whisper-cli-$TRIPLE"

echo "==> Installing model ggml-$MODEL.bin"
mkdir -p "$MODEL_DIR"
if [ -f "$MODEL_DIR/ggml-$MODEL.bin" ]; then
  echo "    already present, skipping download"
else
  curl -L --fail --progress-bar \
    -o "$MODEL_DIR/ggml-$MODEL.bin" \
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-$MODEL.bin"
fi

echo
echo "Done."
echo "  sidecar: $BINARIES/whisper-cli-$TRIPLE"
echo "  model:   $MODEL_DIR/ggml-$MODEL.bin"
