#!/usr/bin/env bash
# Demo 8 — “Federation / kill a node” (product follow-on network polish)
#
# Runs the multi-hop + survivor integration test that:
#   1. Creates a 3-node cluster root
#   2. Serves two nodes over TCP with endpoints.json advertise
#   3. Client directory multi-hop routes keyed put/get to the leader
#   4. Kills one server; the other remains reachable; offline node store is intact
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "== multi-hop route + kill-node (residuum-sdk integration) =="
cargo test -p residuum-sdk --features cluster --test stage8d_routing \
  multi_hop_and_kill_node_survivor -- --nocapture

echo
echo "Also available (experimental routing only; not network quorum):"
echo "  residuum serve-cluster CLUSTER --node N --bind 127.0.0.1:PORT --experimental-network-cluster"
echo "Offline salvage of a dead node: residuum doctor / residuum salvage on nodes/node-N"
echo "demo complete"
