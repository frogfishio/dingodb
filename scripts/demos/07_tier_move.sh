#!/usr/bin/env bash
# Demo 7 — “Keep it fifteen years” (Stage 9 / human milestone §8.7)
#
# Put durable data, seal via high-volume path is awkward from CLI alone; this
# script uses a short Rust one-shot through cargo test helpers style: prefer
# the library APIs when CLI has no `tier` subcommand yet.
#
# Until CLI grows `residiuum tier move`, we document the operator story and run
# the Stage 9 unit suite as the living acceptance gate.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "== Stage 9 living acceptance (cargo test stage9_tiering) =="
cargo test -q -p residiuum-store --test stage9_tiering

echo
echo "Operator story (Rust API / runbook):"
echo "  1. Seal active segment"
echo "  2. Store::transfer_segment_to_tier(id, TierClass::Cold|Archive, Move|Copy)"
echo "  3. Offline archive → tier_coverage incomplete (not empty success)"
echo "  4. Media roots in tiers/roots.txt may be filesystem or object:local:… URIs"
echo "  See doc/reference/operations/RUNBOOK_RETENTION.md"
echo "demo complete"
