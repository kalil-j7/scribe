#!/usr/bin/env bash
# Install the scribe release binary onto PATH and register it in ~/.zshrc.
#
#   ./scripts/install.sh
#
# What it does:
#   1. Builds the release binary if missing or stale.
#   2. Installs it to ~/.local/bin/scribe  (~/.local/bin is already on PATH).
#   3. Appends a guarded, idempotent block to ~/.zshrc that ensures
#      ~/.local/bin is on PATH and defines `scribe-update` to rebuild and
#      reinstall after `git pull` / code changes.
#
# Run `source ~/.zshrc` (or open a new terminal) afterwards.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
INSTALLED="$BIN_DIR/scribe"
ZSHRC="$HOME/.zshrc"

if [[ ! -x "$REPO_DIR/target/release/scribe" ]] || \
   [[ "$REPO_DIR/Cargo.toml" -nt "$REPO_DIR/target/release/scribe" ]] || \
   [[ "$REPO_DIR/src" -nt "$REPO_DIR/target/release/scribe" ]]; then
  echo "Building release binary…"
  (cd "$REPO_DIR" && cargo build --release)
fi

mkdir -p "$BIN_DIR"
cp "$REPO_DIR/target/release/scribe" "$INSTALLED"
chmod +x "$INSTALLED"
echo "Installed: $INSTALLED"

if [[ -f "$ZSHRC" ]] && grep -q "# >>> scribe >>>" "$ZSHRC"; then
  echo "$ZSHRC already contains the scribe block (not duplicating)."
else
  {
    echo ""
    echo "# >>> scribe >>>"
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo "scribe-update() {"
    echo "  (cd \"$REPO_DIR\" && cargo build --release) && \\"
    echo "    cp \"$REPO_DIR/target/release/scribe\" \"\$HOME/.local/bin/scribe\""
    echo "}"
    echo "# <<< scribe <<<"
  } >> "$ZSHRC"
  echo "Appended scribe block to $ZSHRC"
fi

if command -v scribe >/dev/null 2>&1; then
  echo ""
  echo "scribe is ready: $(command -v scribe)"
  echo "Run \`source ~/.zshrc\` (or open a new terminal), then: scribe sirach 2:1"
else
  echo ""
  echo "Done. Open a new terminal or run: source ~/.zshrc"
  echo "Then try: scribe sirach 2:1"
fi
