#!/usr/bin/env bash
# DEF-003 — release-content gate.
# Run from the repository root: ./scripts/release_content.sh
#
# 1. Clean git work tree (release CI must not ship uncommitted slices).
# 2. cargo package --list for every workspace member.
# 3. Rebuild the workspace from package file lists only.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export RUSTFLAGS="${RUSTFLAGS:--D warnings}"

# package_name|member_path (directory under repo root)
MEMBERS=(
  "residuum-sda|crates/sda-core"
  "residuum-sda-cli|crates/sda-cli"
  "residuum-format|crates/residuum-format"
  "residuum-heap|crates/residuum-heap"
  "residuum-store|crates/residuum-store"
  "residuum-client|crates/residuum-client"
  "residuum-sdk|crates/residuum-sdk"
  "residuum-server|crates/residuum-server"
  "residuum-examine|crates/residuum-examine"
  "residuum-cli|crates/residuum-cli"
  "residuum-cluster|crates/residuum-cluster"
  # Workspace member (publish = false); still required for staged rebuild from package lists.
  "residuum-testrig|crates/residuum-testrig"
)

ALLOW_DIRTY="${RESIDUUM_RELEASE_ALLOW_DIRTY:-0}"

echo "== git status (must be clean) =="
if [[ -n "$(git status --short)" ]]; then
  git status --short
  if [[ "$ALLOW_DIRTY" == "1" ]]; then
    echo "warning: dirty work tree allowed via RESIDUUM_RELEASE_ALLOW_DIRTY=1"
  else
    echo "error: git work tree is not clean (DEF-003). Commit or stash first," >&2
    echo "       or set RESIDUUM_RELEASE_ALLOW_DIRTY=1 for a local dry-run." >&2
    exit 1
  fi
else
  echo "clean"
fi

echo
echo "== cargo package --list (all workspace packages) =="
for entry in "${MEMBERS[@]}"; do
  pkg="${entry%%|*}"
  path="${entry#*|}"
  echo "--- $pkg ($path) ---"
  list="$(cargo package -p "$pkg" --list --allow-dirty 2>&1)" || {
    echo "$list" >&2
    echo "error: cargo package --list failed for $pkg" >&2
    exit 1
  }
  printf '%s\n' "$list" | sed 's/^/  /'

  # Required entries every crate must ship.
  for required in Cargo.toml README.md; do
    if ! printf '%s\n' "$list" | grep -qx "$required"; then
      echo "error: $pkg package list missing required file: $required" >&2
      exit 1
    fi
  done
  if ! printf '%s\n' "$list" | grep -qE '^src/'; then
    echo "error: $pkg package list has no src/ files" >&2
    exit 1
  fi
  # residuum-store ships the crash matrix as package data.
  if [[ "$pkg" == "residuum-store" ]]; then
    if ! printf '%s\n' "$list" | grep -qx "crash_matrix.v1.json"; then
      echo "error: residuum-store package list missing crash_matrix.v1.json" >&2
      exit 1
    fi
  fi
done

echo
echo "== build from packaged file lists =="
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/residuum-release-content.XXXXXX")"
cleanup() {
  rm -rf "$STAGE"
}
trap cleanup EXIT

# Root workspace skeleton (not produced by per-crate package).
mkdir -p "$STAGE"
cp "$ROOT/Cargo.toml" "$STAGE/Cargo.toml"
cp "$ROOT/Cargo.lock" "$STAGE/Cargo.lock"
cp "$ROOT/LICENSE" "$STAGE/LICENSE"
[[ -f "$ROOT/VERSION" ]] && cp "$ROOT/VERSION" "$STAGE/VERSION"
[[ -f "$ROOT/BUILD" ]] && cp "$ROOT/BUILD" "$STAGE/BUILD"
if [[ -d "$ROOT/.cargo" ]]; then
  mkdir -p "$STAGE/.cargo"
  cp -R "$ROOT/.cargo/." "$STAGE/.cargo/"
fi

for entry in "${MEMBERS[@]}"; do
  pkg="${entry%%|*}"
  path="${entry#*|}"
  dest="$STAGE/$path"
  mkdir -p "$dest"

  # File list relative to the package root (crate directory).
  while IFS= read -r rel; do
    case "$rel" in
      .cargo_vcs_info.json|Cargo.lock|Cargo.toml.orig) continue ;;
      "") continue ;;
    esac
    src="$ROOT/$path/$rel"
    if [[ ! -f "$src" ]]; then
      # cargo sometimes lists files only present after packaging; skip generated.
      if [[ "$rel" == "Cargo.toml" ]]; then
        src="$ROOT/$path/Cargo.toml"
      else
        echo "error: $pkg lists $rel but file missing at $src" >&2
        exit 1
      fi
    fi
    mkdir -p "$(dirname "$dest/$rel")"
    cp "$src" "$dest/$rel"
  done < <(cargo package -p "$pkg" --list --allow-dirty | sed '/^\s*$/d')
done

# Human demos and release policy docs are workspace artifacts (not crate tarballs).
mkdir -p "$STAGE/scripts/demos" "$STAGE/doc"
cp "$ROOT/scripts/demos/"*.sh "$STAGE/scripts/demos/" 2>/dev/null || true
cp "$ROOT/scripts/demos/README.md" "$STAGE/scripts/demos/" 2>/dev/null || true
cp "$ROOT/scripts/release_content.sh" "$STAGE/scripts/" 2>/dev/null || true
cp "$ROOT/doc/RELEASE_ARTIFACTS.md" "$STAGE/doc/" 2>/dev/null || true

(
  cd "$STAGE"
  echo "staging tree at $STAGE"
  cargo build --workspace --all-targets
)

echo
echo "release_content OK"
