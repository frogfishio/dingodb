#!/usr/bin/env bash
# Download a pinned Verus binary release into tools/verus (gitignored).
# macOS arm64 / x86_64 and Linux x86_64 supported for CI/local.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Pin to a known-good weekly release (update deliberately).
VERUS_VER="${RESIDIUUM_VERUS_VERSION:-0.2026.07.27.31579f0}"
OS=$(uname -s)
ARCH=$(uname -m)
case "$OS-$ARCH" in
  Darwin-arm64) ZIP="verus-${VERUS_VER}-arm64-macos.zip" ;;
  Darwin-x86_64) ZIP="verus-${VERUS_VER}-x86-macos.zip" ;;
  Linux-x86_64) ZIP="verus-${VERUS_VER}-x86-linux.zip" ;;
  *)
    echo "setup_verus: unsupported platform $OS-$ARCH" >&2
    exit 1
    ;;
esac

mkdir -p "$ROOT/tools"
cd "$ROOT/tools"
URL="https://github.com/verus-lang/verus/releases/download/release%2F${VERUS_VER}/${ZIP}"
echo "setup_verus: fetching $URL"
curl -fsSL -o verus.zip "$URL"
rm -rf verus verus-arm64-macos verus-x86-macos verus-x86-linux
unzip -qo verus.zip
# Normalize extract dir name → tools/verus
EXTRACTED=$(find . -maxdepth 2 -type f -name verus | head -1)
[[ -n "$EXTRACTED" ]] || { echo "setup_verus: verus binary not found in zip" >&2; exit 1; }
mv "$(dirname "$EXTRACTED")" verus
rm -f verus.zip
if [[ "$OS" == "Darwin" ]]; then
  xattr -dr com.apple.quarantine verus 2>/dev/null || true
fi
# Install required rustup toolchain if verus reports it missing.
if ! ./verus/verus --version >/dev/null 2>&1; then
  NEED=$(./verus/verus 2>&1 | sed -n 's/.*rustup install //p' | head -1 | awk '{print $1}')
  if [[ -n "$NEED" ]]; then
    echo "setup_verus: installing rust toolchain $NEED"
    rustup install "$NEED"
  fi
fi
./verus/verus --version
echo "setup_verus: OK → $ROOT/tools/verus/verus"
