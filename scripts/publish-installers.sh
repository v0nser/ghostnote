#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/website/public/downloads"
mkdir -p "$DEST"

copy_if_exists() {
  local src="$1"
  local name="$2"
  if [[ -f "$src" ]]; then
    cp "$src" "$DEST/$name"
    echo "Published $name ($(du -h "$DEST/$name" | awk '{print $1}'))"
  fi
}

shopt -s nullglob
SEARCH=(
  "$ROOT"/src-tauri/target/release/bundle/dmg/*.dmg
  "$ROOT"/src-tauri/target/*/release/bundle/dmg/*.dmg
)
if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  SEARCH+=("$CARGO_TARGET_DIR"/release/bundle/dmg/*.dmg)
fi
for dmg in "${SEARCH[@]}"; do
  copy_if_exists "$dmg" "GhostNote.dmg"
done
for exe in "$ROOT"/src-tauri/target/release/bundle/nsis/*setup.exe; do
  copy_if_exists "$exe" "GhostNote-Setup.exe"
done
for exe in "$ROOT"/src-tauri/target/*/release/bundle/nsis/*setup.exe; do
  copy_if_exists "$exe" "GhostNote-Setup.exe"
done

cat > "$DEST/latest.json" <<EOF
{
  "version": "0.1.0",
  "mac": $([ -f "$DEST/GhostNote.dmg" ] && echo '{"available":true,"url":"/downloads/GhostNote.dmg","filename":"GhostNote.dmg"}' || echo '{"available":false}'),
  "windows": $([ -f "$DEST/GhostNote-Setup.exe" ] && echo '{"available":true,"url":"/downloads/GhostNote-Setup.exe","filename":"GhostNote-Setup.exe"}' || echo '{"available":false}')
}
EOF

echo "Wrote $DEST/latest.json"
ls -lh "$DEST"
