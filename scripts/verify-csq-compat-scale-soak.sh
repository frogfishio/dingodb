#!/usr/bin/env bash
# CSQ-11: compatibility matrix floors + packaged journey + PR-safe scale/soak seed.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-11 compatibility / scale / soak suite..."
cargo test -p residiuum-store --features legacy-raw-store --test csq11_compat_scale_soak -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-compat-scale-soak: FAILED" >&2
  exit 1
fi
echo "verify-csq-compat-scale-soak: OK"
echo "  suite: csq11_compat_scale_soak (COMPAT + journey + PR-safe scale seed; 24h/72h residual)"
