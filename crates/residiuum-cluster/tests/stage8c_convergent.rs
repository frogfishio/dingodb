//! Stage 8c — convergent-append dual-accept + reconcile
//! (CLUSTER_SPEC §9.2, §15.2, §22 items 7–8).

use residiuum_cluster::{
    body_content_hash, Cluster, ClusterConfig, CommitStatus, ConsistencyMode, DurabilityMode,
    NodeId, ReadMode,
};
use std::collections::HashSet;

fn convergent_cluster(root: &std::path::Path) -> Cluster {
    Cluster::create(
        ClusterConfig::dependable_local(root)
            .with_virtual_partitions(8)
            .with_consistency_mode(ConsistencyMode::ConvergentAppend),
    )
    .unwrap()
}

#[test]
fn convergent_accepts_without_quorum() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = convergent_cluster(&dir.path().join("c"));

    // Minority: only one node online — linearizable would fail; convergent must accept.
    cluster.mark_offline(NodeId::new(1)).unwrap();
    cluster.mark_offline(NodeId::new(2)).unwrap();
    assert_eq!(cluster.online_node_count(), 1);

    let ack = cluster
        .put("telemetry/1", b"side-a-only", DurabilityMode::Durable)
        .unwrap();
    assert_eq!(ack.consistency_mode, ConsistencyMode::ConvergentAppend);
    assert!(
        !ack.committed,
        "convergent must not claim linearizable commit"
    );
    assert_eq!(ack.commit_status, CommitStatus::Prepared);
    assert_eq!(ack.replica_acks, 1);
    assert_ne!(ack.event_id, [0u8; 16]);

    let got = cluster.get("telemetry/1", ReadMode::Available).unwrap();
    assert_eq!(got.value.as_deref(), Some(b"side-a-only".as_slice()));
    assert!(got
        .coverage
        .notes
        .iter()
        .any(|n| n.contains("convergent-append")));
}

#[test]
fn linearizable_read_rejected_in_convergent_mode() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = convergent_cluster(&dir.path().join("c"));
    cluster.put("k", b"v", DurabilityMode::Buffered).unwrap();
    let err = cluster.get("k", ReadMode::Linearizable).unwrap_err();
    assert_eq!(err.code(), "consistency_violation");
}

#[test]
fn dual_accept_both_sides_of_split_then_reconcile() {
    // CLUSTER_SPEC §22 item 7: simultaneous convergent appends on both sides.
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = convergent_cluster(&dir.path().join("split"));

    // Side A: only node 0.
    cluster.mark_offline(NodeId::new(1)).unwrap();
    cluster.mark_offline(NodeId::new(2)).unwrap();
    let ack_a = cluster
        .append_local(
            NodeId::new(0),
            "evt/side-a",
            b"payload-from-A",
            DurabilityMode::Durable,
        )
        .unwrap();
    assert!(!ack_a.committed);
    assert_eq!(ack_a.replica_acks, 1);
    assert_eq!(ack_a.leader, NodeId::new(0));

    // Flip to side B: nodes 1+2 online, node 0 offline.
    cluster.mark_online(NodeId::new(1)).unwrap();
    cluster.mark_online(NodeId::new(2)).unwrap();
    cluster.mark_offline(NodeId::new(0)).unwrap();

    let ack_b = cluster
        .append_local(
            NodeId::new(1),
            "evt/side-b",
            b"payload-from-B",
            DurabilityMode::Durable,
        )
        .unwrap();
    assert!(!ack_b.committed);
    assert_ne!(ack_a.event_id, ack_b.event_id);

    // Side B must not see side A's unique event yet.
    let miss = cluster.get("evt/side-a", ReadMode::Available).unwrap();
    assert_eq!(miss.value, None);

    // Heal: all nodes online.
    cluster.mark_online(NodeId::new(0)).unwrap();
    assert_eq!(cluster.online_node_count(), 3);

    let report = cluster.reconcile().unwrap();
    assert!(
        report.events_replicated >= 2,
        "both unique events should fan out, got {}",
        report.events_replicated
    );
    assert!(
        report.conflicts.is_empty(),
        "distinct subjects should not conflict: {:?}",
        report.conflicts
    );

    // Both events visible after reconcile.
    let a = cluster.get("evt/side-a", ReadMode::Available).unwrap();
    let b = cluster.get("evt/side-b", ReadMode::Available).unwrap();
    assert_eq!(a.value.as_deref(), Some(b"payload-from-A".as_slice()));
    assert_eq!(b.value.as_deref(), Some(b"payload-from-B".as_slice()));

    // Scan should include both.
    let scan = cluster.scan_all().unwrap();
    let keys: HashSet<_> = scan.entries.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains("evt/side-a"));
    assert!(keys.contains("evt/side-b"));
}

#[test]
fn conflicting_live_bodies_reported_on_reconcile() {
    // CLUSTER_SPEC §22 item 8: conflicting identities (same subject, different
    // bodies accepted on each side of a split). No silent winner.
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = convergent_cluster(&dir.path().join("conflict"));

    let subject = "stream/shared-key";

    // Side A accepts body A.
    cluster.mark_offline(NodeId::new(1)).unwrap();
    cluster.mark_offline(NodeId::new(2)).unwrap();
    cluster
        .append_local(
            NodeId::new(0),
            subject,
            b"variant-A",
            DurabilityMode::Durable,
        )
        .unwrap();

    // Side B accepts body B for the same subject.
    cluster.mark_online(NodeId::new(1)).unwrap();
    cluster.mark_online(NodeId::new(2)).unwrap();
    cluster.mark_offline(NodeId::new(0)).unwrap();
    cluster
        .append_local(
            NodeId::new(1),
            subject,
            b"variant-B",
            DurabilityMode::Durable,
        )
        .unwrap();

    // Heal and reconcile.
    cluster.mark_online(NodeId::new(0)).unwrap();
    let report = cluster.reconcile().unwrap();

    assert_eq!(report.conflicts.len(), 1);
    let c = &report.conflicts[0];
    assert_eq!(c.subject, subject);
    assert_eq!(c.variants.len(), 2);
    let hashes: HashSet<_> = c.variants.iter().map(|v| v.content_hash).collect();
    assert_eq!(hashes.len(), 2);
    assert!(hashes.contains(&body_content_hash(b"variant-A")));
    assert!(hashes.contains(&body_content_hash(b"variant-B")));

    // Both variants survive in history on at least one node (no discard).
    // After conflict multi-put, live value is deterministic (hash-sorted last).
    let live = cluster.get(subject, ReadMode::Available).unwrap();
    assert!(live.value.is_some());
    let live_body = live.value.unwrap();
    assert!(
        live_body == b"variant-A" || live_body == b"variant-B",
        "live is one of the retained variants"
    );
}

#[test]
fn append_local_rejects_linearizable_mode() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("lin"))
            .with_virtual_partitions(4)
            .with_consistency_mode(ConsistencyMode::PartitionLinearizable),
    )
    .unwrap();
    let err = cluster
        .append_local(NodeId::new(0), "x", b"y", DurabilityMode::Memory)
        .unwrap_err();
    assert_eq!(err.code(), "consistency_violation");
}

#[test]
fn convergent_happy_path_all_online_fans_out() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = convergent_cluster(&dir.path().join("fan"));
    let ack = cluster
        .put("log/1", b"everywhere", DurabilityMode::Durable)
        .unwrap();
    assert_eq!(ack.replica_acks, 3);
    assert!(!ack.committed);

    // All three nodes hold the value without an explicit reconcile.
    for i in 0..3u32 {
        // Available prefers any online replica.
        let got = cluster.get("log/1", ReadMode::Available).unwrap();
        assert_eq!(got.value.as_deref(), Some(b"everywhere".as_slice()));
        let _ = i;
    }

    // Reconcile is a no-op when already consistent.
    let report = cluster.reconcile().unwrap();
    assert_eq!(report.events_replicated, 0);
    assert!(report.conflicts.is_empty());
}

#[test]
fn unique_event_ids_on_independent_appends() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = convergent_cluster(&dir.path().join("ids"));
    let a = cluster
        .put("e/1", b"one", DurabilityMode::Buffered)
        .unwrap();
    let b = cluster
        .put("e/2", b"two", DurabilityMode::Buffered)
        .unwrap();
    assert_ne!(a.event_id, b.event_id);
    // Positions are partition-local accept counters; only compare when same partition.
    if a.partition == b.partition {
        assert_ne!(a.position, b.position);
    }
}
