//! Stage 8a cluster foundation tests (CLUSTER_SPEC §6, §8–§11, §15, §22 subset).
//!
//! Physical survival, logical commitment, and query coverage are asserted as
//! separate claims.

use dingo_cluster::{
    Cluster, ClusterConfig, ConsistencyMode, DeploymentProfile, DurabilityMode, NodeId,
    PartitionMap, ReadMode, HASH_PROFILE_BLAKE3_MOD,
};
use dingo_store::Store;
use std::collections::HashSet;

#[test]
fn development_profile_put_get_with_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("dev");
    let mut cluster = Cluster::create(
        ClusterConfig::development(&root)
            .with_virtual_partitions(8)
            .with_consistency_mode(ConsistencyMode::PartitionLinearizable),
    )
    .unwrap();

    assert_eq!(cluster.profile(), DeploymentProfile::Development);
    assert!(!cluster.replicated_durability_available());
    assert_eq!(cluster.write_quorum(), 1);
    assert_eq!(cluster.online_node_count(), 1);

    let ack = cluster
        .put("users/alice", br#"{"name":"Alice"}"#, DurabilityMode::Durable)
        .unwrap();
    assert!(ack.committed);
    assert_eq!(ack.replica_acks, 1);
    assert_eq!(ack.consistency_mode, ConsistencyMode::PartitionLinearizable);
    assert_eq!(ack.durability_mode, DurabilityMode::Durable);

    let got = cluster
        .get("users/alice", ReadMode::Linearizable)
        .unwrap();
    assert_eq!(got.value.as_deref(), Some(br#"{"name":"Alice"}"#.as_slice()));
    assert!(got.coverage.is_complete());
    assert!(got.absence_proven);
    assert!(got
        .coverage
        .notes
        .iter()
        .any(|n| n.contains("replicated durability unavailable")));
}

#[test]
fn partition_map_is_stable_and_published() {
    let map = PartitionMap::new(32);
    assert_eq!(map.hash_profile, HASH_PROFILE_BLAKE3_MOD);
    let a = map.partition_of(b"subject-1");
    let b = map.partition_of(b"subject-1");
    assert_eq!(a, b);
    assert!(a.get() < 32);
}

#[test]
fn dependable_local_quorum_replication() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("ha");
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(&root).with_virtual_partitions(16),
    )
    .unwrap();

    assert_eq!(cluster.profile(), DeploymentProfile::DependableLocal);
    assert_eq!(cluster.online_node_count(), 3);
    assert_eq!(cluster.write_quorum(), 2);
    assert!(cluster.replicated_durability_available());

    let ack = cluster
        .put("orders/1", b"payload-v1", DurabilityMode::Durable)
        .unwrap();
    assert!(ack.committed);
    assert!(ack.replica_acks >= 2);
    assert_eq!(ack.replica_acks, 3); // all three online

    // Linearizable read from leader.
    let got = cluster.get("orders/1", ReadMode::Linearizable).unwrap();
    assert_eq!(got.value.as_deref(), Some(b"payload-v1".as_slice()));
    assert!(got.coverage.is_complete());

    // Followers also hold the value (balanced full-replica placement).
    for i in 0..3 {
        let path = cluster.node_path(NodeId::new(i));
        // Don't open while cluster holds handles — check via cluster get available
        // after marking leader offline later; here just ensure paths exist.
        assert!(path.exists(), "node-{i} path");
    }
}

#[test]
fn leader_offline_triggers_reelection_when_quorum_remains() {
    // Stage 8b: static primary is gone — a live majority elects a new leader.
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(8),
    )
    .unwrap();

    let subject = "k/only-one-partition";
    let partition = cluster.partition_for_subject(subject);
    // Seed a leader so directory/raft have a known primary, then kill it.
    let ack0 = cluster
        .put(subject, b"seed", DurabilityMode::Buffered)
        .unwrap();
    let old_leader = ack0.leader;
    cluster.mark_offline(old_leader).unwrap();
    assert!(!cluster.is_online(old_leader));
    assert_eq!(cluster.online_node_count(), 2);

    let ack = cluster
        .put(subject, b"x", DurabilityMode::Buffered)
        .unwrap();
    assert!(ack.committed);
    assert_ne!(ack.leader, old_leader);
    assert!(ack.term.0 >= ack0.term.0);

    let got = cluster.get(subject, ReadMode::Linearizable).unwrap();
    assert_eq!(got.value.as_deref(), Some(b"x".as_slice()));
    let _ = partition;
}

#[test]
fn available_read_survives_leader_loss_if_replica_holds_data() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(8),
    )
    .unwrap();

    let subject = "session/xyz";
    cluster
        .put(subject, b"live", DurabilityMode::Durable)
        .unwrap();
    let partition = cluster.partition_for_subject(subject);
    let leader = cluster.directory().leader_of(partition).unwrap();

    cluster.mark_offline(leader).unwrap();

    // Available mode may return from a follower.
    let got = cluster.get(subject, ReadMode::Available).unwrap();
    assert_eq!(got.value.as_deref(), Some(b"live".as_slice()));
    assert!(got.coverage.is_complete());
    assert!(!got.absence_proven);
}

#[test]
fn scan_marks_unavailable_partitions_not_empty_success() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(6),
    )
    .unwrap();

    // Write enough keys to touch multiple partitions.
    for i in 0..30 {
        let key = format!("item/{i}");
        cluster
            .put(&key, format!("v{i}").as_bytes(), DurabilityMode::Buffered)
            .unwrap();
    }

    // Take down one whole node — any partition whose *only* remaining path was
    // that node still has full replicas, so scan may still complete. Take down
    // two nodes so some partitions lose all online replicas if leader and both
    // followers are the same three — with full replication of all partitions
    // to all nodes, one online node can still serve every partition.
    //
    // Mark two offline so only one remains — scan should still complete
    // (remaining node holds all partitions). Mark all three offline → full holes.
    cluster.mark_offline(NodeId::new(0)).unwrap();
    cluster.mark_offline(NodeId::new(1)).unwrap();
    let scan = cluster.scan_all().unwrap();
    assert!(scan.coverage.is_complete());
    assert!(!scan.entries.is_empty());

    cluster.mark_offline(NodeId::new(2)).unwrap();
    let scan2 = cluster.scan_all().unwrap();
    assert!(scan2.coverage.is_incomplete());
    assert_eq!(scan2.coverage.unavailable.len(), 6);
    // Incomplete must not be presented as "empty success".
    assert!(scan2.coverage.completed.is_empty());
}

#[test]
fn quorum_loss_rejects_strong_write() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(4),
    )
    .unwrap();

    // With only one online node, replica_acks max is 1 < quorum 2.
    cluster.mark_offline(NodeId::new(1)).unwrap();
    cluster.mark_offline(NodeId::new(2)).unwrap();

    // Leader for a given key may still be node 0. If leader is offline, we get
    // partition_unavailable; if leader is the remaining node, durability fails.
    let mut saw_expected = false;
    for i in 0..20 {
        let key = format!("q/{i}");
        match cluster.put(&key, b"z", DurabilityMode::Durable) {
            Ok(_) => panic!("expected strong write to fail under minority"),
            Err(e) => {
                let c = e.code();
                assert!(
                    c == "durability_unavailable" || c == "partition_unavailable",
                    "unexpected code {c}"
                );
                saw_expected = true;
            }
        }
    }
    assert!(saw_expected);
}

#[test]
fn node_salvage_without_cluster_software() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("c");
    let mut cluster =
        Cluster::create(ClusterConfig::development(&root).with_virtual_partitions(4)).unwrap();

    cluster
        .put("salvage/me", b"important", DurabilityMode::Durable)
        .unwrap();

    let node_path = cluster.node_path(NodeId::new(0));

    // Drop the cluster handle entirely — salvage with only dingo-store.
    drop(cluster);

    let report = Cluster::salvage_node_path(&node_path).unwrap();
    assert!(report.verified_frames > 0);
    assert!(report.item_events > 0);
    assert!(report.live_subjects >= 1);

    // Also open as a plain store and get the value.
    let store = Store::open(&node_path).unwrap();
    let body = store.get("salvage/me").unwrap();
    assert_eq!(body.as_deref(), Some(b"important".as_slice()));
}

#[test]
fn open_roundtrip_preserves_data() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("persist");
    {
        let mut c = Cluster::create(ClusterConfig::development(&root).with_virtual_partitions(4))
            .unwrap();
        c.put("a", b"1", DurabilityMode::Durable).unwrap();
    }
    let mut c = Cluster::open(&root).unwrap();
    let got = c.get("a", ReadMode::Linearizable).unwrap();
    assert_eq!(got.value.as_deref(), Some(b"1".as_slice()));
}

#[test]
fn delete_removes_live_value() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Cluster::create(
        ClusterConfig::development(dir.path().join("d")).with_virtual_partitions(4),
    )
    .unwrap();
    c.put("x", b"y", DurabilityMode::Durable).unwrap();
    let ack = c.delete("x", DurabilityMode::Durable).unwrap();
    assert!(ack.committed);
    let got = c.get("x", ReadMode::Linearizable).unwrap();
    assert_eq!(got.value, None);
    assert!(got.absence_proven);
}

#[test]
fn keys_spread_across_partitions() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Cluster::create(
        ClusterConfig::development(dir.path().join("d")).with_virtual_partitions(8),
    )
    .unwrap();
    let mut parts = HashSet::new();
    for i in 0..40 {
        let key = format!("spread/{i}");
        let ack = c.put(&key, b"v", DurabilityMode::Memory).unwrap();
        parts.insert(ack.partition.get());
    }
    assert!(parts.len() >= 4, "expected key spread, got {parts:?}");
}
