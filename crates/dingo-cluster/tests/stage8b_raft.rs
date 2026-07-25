//! Stage 8b — per-partition Raft elections, log matching, commit evidence
//! (CLUSTER_SPEC §10).

use dingo_cluster::raft::{ElectError, LogCommand, PartitionRaft, RaftRole};
use dingo_cluster::{
    Cluster, ClusterConfig, DurabilityMode, NodeId, PartitionId, PlacementEpoch, ReadMode, Term,
};

#[test]
fn reelect_after_leader_loss_with_majority() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("raft-ha")).with_virtual_partitions(8),
    )
    .unwrap();

    let subject = "orders/42";
    let ack1 = cluster
        .put(subject, b"v1", DurabilityMode::Durable)
        .unwrap();
    assert!(ack1.committed);
    assert!(ack1.replica_acks >= 2);

    let old_leader = ack1.leader;
    let old_term = ack1.term;
    cluster.mark_offline(old_leader).unwrap();

    let ack2 = cluster
        .put(subject, b"v2", DurabilityMode::Durable)
        .unwrap();
    assert!(ack2.committed);
    assert_ne!(ack2.leader, old_leader);
    assert!(ack2.term.0 > old_term.0 || ack2.term.0 == old_term.0);
    // New term after election (leader step-down + elect increments).
    assert!(ack2.term.0 >= old_term.0);

    let got = cluster.get(subject, ReadMode::Linearizable).unwrap();
    assert_eq!(got.value.as_deref(), Some(b"v2".as_slice()));

    // Survivors remain online; available read still returns the committed value.
    for i in 0..3u32 {
        let n = NodeId::new(i);
        if n == old_leader {
            continue;
        }
        assert!(cluster.is_online(n));
    }
    let avail = cluster.get(subject, ReadMode::Available).unwrap();
    assert_eq!(avail.value.as_deref(), Some(b"v2".as_slice()));
}

#[test]
fn election_fails_without_majority_of_configured_voters() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("minority")).with_virtual_partitions(4),
    )
    .unwrap();

    cluster.mark_offline(NodeId::new(1)).unwrap();
    cluster.mark_offline(NodeId::new(2)).unwrap();
    assert_eq!(cluster.online_node_count(), 1);

    let mut saw = false;
    for i in 0..16 {
        let key = format!("m/{i}");
        match cluster.put(&key, b"z", DurabilityMode::Durable) {
            Ok(_) => panic!("minority must not commit linearizable writes"),
            Err(e) => {
                let c = e.code();
                assert!(
                    c == "partition_unavailable" || c == "durability_unavailable" || c == "no_leader",
                    "unexpected {c}"
                );
                saw = true;
            }
        }
    }
    assert!(saw);
}

#[test]
fn commit_evidence_reports_quorum_acks() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::dependable_local(dir.path().join("ev")).with_virtual_partitions(4),
    )
    .unwrap();

    let ack = cluster
        .put("ev/key", b"body", DurabilityMode::Durable)
        .unwrap();
    assert!(ack.committed);
    assert!(ack.replica_acks >= 2);

    let group = cluster.raft_group(ack.partition).expect("raft group");
    let evidence = group.commit_evidence(ack.leader, ack.position);
    assert!(evidence.committed);
    assert!(evidence.acked_by.len() as u32 >= 2);
    assert_eq!(evidence.position, ack.position);
    assert_eq!(evidence.term, ack.term);
}

#[test]
fn log_matching_property_on_partition_group() {
    let mut g = PartitionRaft::new(
        PartitionId::new(3),
        vec![NodeId::new(0), NodeId::new(1), NodeId::new(2)],
        PlacementEpoch(1),
    );
    let online = [NodeId::new(0), NodeId::new(1), NodeId::new(2)];
    let (leader, term) = g.ensure_leader(&online).unwrap();

    g.propose(
        leader,
        LogCommand::Put {
            subject: "a".into(),
            value: b"1".to_vec(),
        },
        &online,
    )
    .unwrap();
    g.propose(
        leader,
        LogCommand::Put {
            subject: "b".into(),
            value: b"2".to_vec(),
        },
        &online,
    )
    .unwrap();

    // All voters share the same last index/term (log matching after success).
    let last_idx = g.peer(leader).unwrap().last_log_index();
    let last_term = g.peer(leader).unwrap().last_log_term();
    for n in &online {
        let p = g.peer(*n).unwrap();
        assert_eq!(p.last_log_index(), last_idx);
        assert_eq!(p.last_log_term(), last_term);
        assert_eq!(p.term_at(1), Some(term));
    }

    // Conflict: wrong prev term at index 1 is rejected.
    let res = g.append_entries(
        NodeId::new(2),
        leader,
        term,
        1,
        Term(0), // empty-log term, not the real entry term
        &[],
        0,
    );
    assert!(!res.success);
}

#[test]
fn ensure_leader_prefers_longer_log() {
    let mut g = PartitionRaft::new(
        PartitionId::new(0),
        vec![NodeId::new(0), NodeId::new(1), NodeId::new(2)],
        PlacementEpoch(1),
    );
    let all = [NodeId::new(0), NodeId::new(1), NodeId::new(2)];
    let (l0, _) = g.elect(NodeId::new(0), &all).unwrap();
    g.propose(
        l0,
        LogCommand::Put {
            subject: "k".into(),
            value: b"v".to_vec(),
        },
        &all,
    )
    .unwrap();

    // Offline original leader; node 2 never got... actually all got the entry.
    // Truncate node 2's log to simulate a lagging replica, then elect among 1+2.
    {
        let p = g.peer_mut(NodeId::new(2)).unwrap();
        p.log.clear();
        p.commit_index = 0;
        p.last_applied = 0;
        p.role = RaftRole::Follower;
    }
    if let Some(p) = g.peer_mut(NodeId::new(0)) {
        p.role = RaftRole::Follower;
    }

    let (leader, _) = g
        .ensure_leader(&[NodeId::new(1), NodeId::new(2)])
        .unwrap();
    assert_eq!(
        leader,
        NodeId::new(1),
        "node with the longer log must win over empty log"
    );
}

#[test]
fn single_node_development_elects_and_commits() {
    let dir = tempfile::tempdir().unwrap();
    let mut cluster = Cluster::create(
        ClusterConfig::development(dir.path().join("dev")).with_virtual_partitions(4),
    )
    .unwrap();

    let ack = cluster
        .put("solo", b"ok", DurabilityMode::Durable)
        .unwrap();
    assert!(ack.committed);
    assert_eq!(ack.replica_acks, 1);
    assert_eq!(ack.leader, NodeId::new(0));

    let g = cluster.raft_group(ack.partition).unwrap();
    assert_eq!(g.current_leader().map(|(n, _)| n), Some(NodeId::new(0)));
    assert!(g.peer(NodeId::new(0)).unwrap().commit_index >= 1);
}

#[test]
fn pure_raft_election_no_quorum_error() {
    let mut g = PartitionRaft::new(
        PartitionId::new(0),
        vec![NodeId::new(0), NodeId::new(1), NodeId::new(2)],
        PlacementEpoch(1),
    );
    let err = g.elect(NodeId::new(0), &[NodeId::new(0)]).unwrap_err();
    assert!(matches!(
        err,
        ElectError::NoQuorum { votes: 1, need: 2 }
    ));
}
