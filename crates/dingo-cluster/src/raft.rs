//! Per-partition Raft-equivalent consensus (Stage 8b, CLUSTER_SPEC §10).
//!
//! This is an in-process, purpose-built protocol with the **safety properties**
//! of Raft (Ongaro & Ousterhout). It is not a full production multi-threaded
//! Raft library; timers are synthetic and RPCs are direct function calls so
//! tests can drive elections and log matching deterministically.
//!
//! # Published rules (CLUSTER_SPEC §10.1)
//!
//! ## Election and terms
//! - Each partition group has a fixed set of voting replicas.
//! - A candidate increments `current_term`, votes for itself, and issues
//!   RequestVote to all voters.
//! - A vote is granted only if the receiver's term is not greater, it has not
//!   voted for another candidate in this term, and the candidate's log is at
//!   least as up-to-date (last entry term, then last index).
//! - Leadership requires votes from a majority of the **configured** voter set
//!   (`floor(N/2)+1`), not merely a majority of currently online nodes.
//! - Discovering a higher term steps down any leader/candidate to follower.
//!
//! ## Log matching and commitment
//! - Log entries are identified by `(term, index)` with 1-based indices.
//! - AppendEntries carries `prev_log_index` / `prev_log_term`; a follower
//!   rejects if its log does not match at that point (log matching property).
//! - On conflict, the follower truncates its log from the first mismatch and
//!   appends the leader's entries.
//! - A leader advances `commit_index` to the highest index `N` such that a
//!   majority of voters have `match_index >= N` **and** `log[N].term ==
//!   current_term` (Raft §5.4.2 — no commit of prior-term entries by count alone).
//! - Followers advance `commit_index` from the leader's `leader_commit`.
//!
//! ## Membership / leases / snapshots
//! - Stage 8b used a fixed voter set from the placement directory.
//! - Stage 8f adds interruptible rebalance membership changes
//!   ([`PartitionRaft::set_voters`]); leader leases and log snapshots remain
//!   out of scope.
//!
//! ## Persistence
//! - Raft state is held in the in-process cluster handle. Physical subject
//!   data remains in ordinary `dingo-store` nodes; salvage does not depend on
//!   this control plane (CLUSTER_SPEC §6).

use crate::id::{LogPosition, NodeId, PartitionId, PlacementEpoch, Term};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kind of client command stored in the consensus log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogCommand {
    /// Put subject → value bytes.
    Put {
        /// Subject key.
        subject: String,
        /// Payload body.
        value: Vec<u8>,
    },
    /// Delete / tombstone subject.
    Delete {
        /// Subject key.
        subject: String,
    },
}

impl LogCommand {
    /// Subject this command mutates.
    pub fn subject(&self) -> &str {
        match self {
            Self::Put { subject, .. } | Self::Delete { subject } => subject,
        }
    }
}

/// One Raft log entry (term + index + command).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Term in which the leader created this entry.
    pub term: Term,
    /// 1-based log index.
    pub index: u64,
    /// Client command.
    pub command: LogCommand,
}

/// Role of a replica in the partition group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaftRole {
    /// Passive replica; accepts AppendEntries from a leader.
    Follower,
    /// Requesting votes for leadership in the current term.
    Candidate,
    /// Authoritative writer for the partition in the current term.
    Leader,
}

/// Per-node Raft volatile and log state for one partition.
#[derive(Debug, Clone)]
pub struct RaftPeer {
    /// This peer's node id.
    pub node_id: NodeId,
    /// Latest term this peer has observed.
    pub current_term: Term,
    /// Candidate voted for in `current_term`, if any.
    pub voted_for: Option<NodeId>,
    /// Current role.
    pub role: RaftRole,
    /// Log entries in index order (entry at vector position `i` has index `i+1`).
    pub log: Vec<LogEntry>,
    /// Highest index known committed.
    pub commit_index: u64,
    /// Highest index applied to the local state machine.
    pub last_applied: u64,
    /// For leaders: next log index to send to each follower (by node index).
    pub next_index: HashMap<u32, u64>,
    /// For leaders: highest log index known replicated on each follower.
    pub match_index: HashMap<u32, u64>,
}

impl RaftPeer {
    fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            current_term: Term(0),
            voted_for: None,
            role: RaftRole::Follower,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    /// Last log index (0 if empty).
    pub fn last_log_index(&self) -> u64 {
        self.log.last().map(|e| e.index).unwrap_or(0)
    }

    /// Term of the last log entry (0 if empty).
    pub fn last_log_term(&self) -> Term {
        self.log.last().map(|e| e.term).unwrap_or(Term(0))
    }

    /// Term at a 1-based index, if present.
    pub fn term_at(&self, index: u64) -> Option<Term> {
        if index == 0 {
            return Some(Term(0));
        }
        self.log.get((index - 1) as usize).map(|e| e.term)
    }

    /// Entry at a 1-based index.
    pub fn entry_at(&self, index: u64) -> Option<&LogEntry> {
        if index == 0 {
            return None;
        }
        self.log.get((index - 1) as usize)
    }

    /// Whether this peer's log is at least as up-to-date as `(last_term, last_index)`.
    ///
    /// Raft: compare last terms; if equal, longer log wins.
    fn log_at_least_as_up_to_date(&self, cand_last_term: Term, cand_last_index: u64) -> bool {
        let my_term = self.last_log_term();
        let my_index = self.last_log_index();
        if cand_last_term.0 != my_term.0 {
            cand_last_term.0 >= my_term.0
        } else {
            cand_last_index >= my_index
        }
    }

    fn become_follower(&mut self, term: Term) {
        self.current_term = term;
        self.role = RaftRole::Follower;
        self.voted_for = None;
        self.next_index.clear();
        self.match_index.clear();
    }

    fn become_leader(&mut self, voters: &[NodeId]) {
        self.role = RaftRole::Leader;
        let next = self.last_log_index() + 1;
        self.next_index.clear();
        self.match_index.clear();
        for v in voters {
            self.next_index.insert(v.index(), next);
            self.match_index.insert(v.index(), 0);
        }
        // Leader has its own log fully matched.
        self.match_index
            .insert(self.node_id.index(), self.last_log_index());
        self.next_index
            .insert(self.node_id.index(), self.last_log_index() + 1);
    }
}

/// Result of a RequestVote RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoteResult {
    /// Term of the receiver (may be higher than the candidate's).
    pub term: Term,
    /// Whether the vote was granted.
    pub vote_granted: bool,
}

/// Result of an AppendEntries RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendResult {
    /// Term of the receiver.
    pub term: Term,
    /// Whether the append was accepted (log matched).
    pub success: bool,
    /// On failure, a hint for the leader's next_index backoff (optional).
    pub conflict_index: Option<u64>,
}

/// Commit evidence for a log index (CLUSTER_SPEC §10.4–§10.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitEvidence {
    /// Partition.
    pub partition: PartitionId,
    /// Term of the committed entry (if any).
    pub term: Term,
    /// Committed log position.
    pub position: LogPosition,
    /// Nodes that acknowledged this index (match_index ≥ position).
    pub acked_by: Vec<NodeId>,
    /// Whether quorum evidence proves commitment.
    pub committed: bool,
}

/// Per-partition Raft group over a fixed voter set.
#[derive(Debug, Clone)]
pub struct PartitionRaft {
    /// Virtual partition this group owns.
    pub partition: PartitionId,
    /// Voting members (placement membership).
    pub voters: Vec<NodeId>,
    /// Placement epoch fencing writes for this assignment.
    pub placement_epoch: PlacementEpoch,
    /// Per-voter Raft state.
    peers: HashMap<u32, RaftPeer>,
}

impl PartitionRaft {
    /// Create a group with empty logs for each voter (initial term 0, followers).
    pub fn new(
        partition: PartitionId,
        voters: Vec<NodeId>,
        placement_epoch: PlacementEpoch,
    ) -> Self {
        let mut peers = HashMap::new();
        for v in &voters {
            peers.insert(v.index(), RaftPeer::new(*v));
        }
        Self {
            partition,
            voters,
            placement_epoch,
            peers,
        }
    }

    /// Ensure a peer slot exists (learner catch-up or membership add).
    pub fn ensure_peer(&mut self, node: NodeId) {
        self.peers
            .entry(node.index())
            .or_insert_with(|| RaftPeer::new(node));
    }

    /// Replace the voting set and placement epoch (Stage 8f rebalance).
    ///
    /// Overlapping voters keep log state. New voters get empty peer state
    /// (callers stream the log first). Peers no longer voting remain in
    /// `peers` until explicitly dropped so safety-window replicas retain
    /// evidence; they are ignored for quorum.
    pub fn set_voters(&mut self, voters: Vec<NodeId>, placement_epoch: PlacementEpoch) {
        for v in &voters {
            self.ensure_peer(*v);
        }
        self.voters = voters;
        self.placement_epoch = placement_epoch;
        // If the former leader is no longer a voter, step it down.
        let voter_set: std::collections::HashSet<u32> =
            self.voters.iter().map(|n| n.index()).collect();
        for (idx, peer) in self.peers.iter_mut() {
            if peer.role == RaftRole::Leader && !voter_set.contains(idx) {
                peer.role = RaftRole::Follower;
            }
        }
    }

    /// Copy the leader's log and commit index onto a destination peer
    /// (rebalance log catch-up). Returns the number of entries installed.
    pub fn stream_log_to(&mut self, leader: NodeId, dest: NodeId) -> u64 {
        self.ensure_peer(dest);
        let Some(leader_peer) = self.peers.get(&leader.index()).cloned() else {
            return 0;
        };
        let last_idx = leader_peer.last_log_index();
        let n = leader_peer.log.len() as u64;
        let leader_is_leader = leader_peer.role == RaftRole::Leader;
        let leader_term = leader_peer.current_term;
        let leader_commit = leader_peer.commit_index;
        let leader_log = leader_peer.log;

        if let Some(dest_peer) = self.peers.get_mut(&dest.index()) {
            dest_peer.log = leader_log;
            dest_peer.current_term = leader_term;
            dest_peer.commit_index = leader_commit;
            dest_peer.role = RaftRole::Follower;
            dest_peer.voted_for = None;
            dest_peer.last_applied = 0;
        }
        if leader_is_leader {
            if let Some(lp) = self.peers.get_mut(&leader.index()) {
                if lp.role == RaftRole::Leader {
                    lp.match_index.insert(dest.index(), last_idx);
                    lp.next_index.insert(dest.index(), last_idx + 1);
                }
            }
        }
        n
    }

    /// Write quorum size for this group.
    pub fn quorum(&self) -> u32 {
        let n = self.voters.len() as u32;
        n / 2 + 1
    }

    /// Current leader if some peer is Leader and its term is current.
    pub fn current_leader(&self) -> Option<(NodeId, Term)> {
        let mut best: Option<(NodeId, Term)> = None;
        for v in &self.voters {
            let p = self.peers.get(&v.index())?;
            if p.role == RaftRole::Leader {
                match best {
                    None => best = Some((*v, p.current_term)),
                    Some((_, t)) if p.current_term.0 > t.0 => best = Some((*v, p.current_term)),
                    _ => {}
                }
            }
        }
        best
    }

    /// Peer state (for tests / diagnostics).
    pub fn peer(&self, node: NodeId) -> Option<&RaftPeer> {
        self.peers.get(&node.index())
    }

    /// Mutable peer state.
    pub fn peer_mut(&mut self, node: NodeId) -> Option<&mut RaftPeer> {
        self.peers.get_mut(&node.index())
    }

    /// Highest commit_index observed on any peer.
    pub fn max_commit_index(&self) -> u64 {
        self.peers.values().map(|p| p.commit_index).max().unwrap_or(0)
    }

    /// Run RequestVote on `voter` for `candidate`.
    fn request_vote(
        &mut self,
        voter: NodeId,
        candidate: NodeId,
        candidate_term: Term,
        last_log_index: u64,
        last_log_term: Term,
    ) -> VoteResult {
        let Some(peer) = self.peers.get_mut(&voter.index()) else {
            return VoteResult {
                term: Term(0),
                vote_granted: false,
            };
        };

        if candidate_term.0 < peer.current_term.0 {
            return VoteResult {
                term: peer.current_term,
                vote_granted: false,
            };
        }

        if candidate_term.0 > peer.current_term.0 {
            peer.become_follower(candidate_term);
        }

        let up_to_date = peer.log_at_least_as_up_to_date(last_log_term, last_log_index);
        let can_vote = peer.voted_for.is_none() || peer.voted_for == Some(candidate);

        if can_vote && up_to_date {
            peer.voted_for = Some(candidate);
            peer.current_term = candidate_term;
            VoteResult {
                term: peer.current_term,
                vote_granted: true,
            }
        } else {
            VoteResult {
                term: peer.current_term,
                vote_granted: false,
            }
        }
    }

    /// Attempt to elect `candidate` among `online` voters.
    ///
    /// Returns `Some((leader, term))` on success. The candidate must be online.
    pub fn elect(
        &mut self,
        candidate: NodeId,
        online: &[NodeId],
    ) -> Result<(NodeId, Term), ElectError> {
        if !self.voters.iter().any(|v| *v == candidate) {
            return Err(ElectError::NotAVoter);
        }
        if !online.iter().any(|n| *n == candidate) {
            return Err(ElectError::CandidateOffline);
        }

        let cand_last_index;
        let cand_last_term;
        let new_term;
        {
            let peer = self
                .peers
                .get_mut(&candidate.index())
                .ok_or(ElectError::NotAVoter)?;
            peer.current_term = Term(peer.current_term.0 + 1);
            peer.role = RaftRole::Candidate;
            peer.voted_for = Some(candidate);
            new_term = peer.current_term;
            cand_last_index = peer.last_log_index();
            cand_last_term = peer.last_log_term();
        }

        let mut votes = 1u32; // self-vote
        let mut saw_higher = None;
        let voters = self.voters.clone();

        for voter in voters {
            if voter == candidate {
                continue;
            }
            // Offline voters cannot grant votes (they do not respond).
            if !online.iter().any(|n| *n == voter) {
                continue;
            }
            let res = self.request_vote(
                voter,
                candidate,
                new_term,
                cand_last_index,
                cand_last_term,
            );
            if res.term.0 > new_term.0 {
                saw_higher = Some(res.term);
                break;
            }
            if res.vote_granted {
                votes += 1;
            }
        }

        if let Some(higher) = saw_higher {
            if let Some(peer) = self.peers.get_mut(&candidate.index()) {
                peer.become_follower(higher);
            }
            return Err(ElectError::HigherTerm(higher));
        }

        if votes >= self.quorum() {
            // Step down any other leaders in older/same terms.
            for v in self.voters.clone() {
                if v == candidate {
                    continue;
                }
                if let Some(p) = self.peers.get_mut(&v.index()) {
                    if p.role == RaftRole::Leader || p.role == RaftRole::Candidate {
                        if p.current_term.0 <= new_term.0 {
                            p.role = RaftRole::Follower;
                        }
                    }
                }
            }
            let voters = self.voters.clone();
            if let Some(peer) = self.peers.get_mut(&candidate.index()) {
                peer.current_term = new_term;
                peer.become_leader(&voters);
            }
            Ok((candidate, new_term))
        } else {
            if let Some(peer) = self.peers.get_mut(&candidate.index()) {
                peer.role = RaftRole::Follower;
            }
            Err(ElectError::NoQuorum { votes, need: self.quorum() })
        }
    }

    /// Ensure there is a live leader among `online` nodes; elect if needed.
    ///
    /// Prefer an existing online leader. Otherwise try candidates in stable
    /// order: highest last-log-term, then highest last-log-index, then lowest
    /// node id (deterministic for tests).
    pub fn ensure_leader(
        &mut self,
        online: &[NodeId],
    ) -> Result<(NodeId, Term), ElectError> {
        if let Some((leader, term)) = self.current_leader() {
            if online.iter().any(|n| *n == leader) {
                return Ok((leader, term));
            }
            // Leader is offline — step it down so a new election can proceed.
            if let Some(p) = self.peers.get_mut(&leader.index()) {
                p.role = RaftRole::Follower;
            }
        }

        if online.is_empty() {
            return Err(ElectError::NoOnlineVoters);
        }

        // Rank candidates by log up-to-date-ness, then node id.
        let mut candidates: Vec<NodeId> = online
            .iter()
            .copied()
            .filter(|n| self.voters.iter().any(|v| v == n))
            .collect();
        if candidates.is_empty() {
            return Err(ElectError::NoOnlineVoters);
        }
        candidates.sort_by(|a, b| {
            let pa = self.peers.get(&a.index()).unwrap();
            let pb = self.peers.get(&b.index()).unwrap();
            match pb.last_log_term().0.cmp(&pa.last_log_term().0) {
                std::cmp::Ordering::Equal => {}
                other => return other,
            }
            match pb.last_log_index().cmp(&pa.last_log_index()) {
                std::cmp::Ordering::Equal => {}
                other => return other,
            }
            a.index().cmp(&b.index())
        });

        let mut last_err = ElectError::NoQuorum {
            votes: 0,
            need: self.quorum(),
        };
        for cand in candidates {
            match self.elect(cand, online) {
                Ok(lt) => return Ok(lt),
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    /// AppendEntries RPC: leader → follower.
    pub fn append_entries(
        &mut self,
        follower: NodeId,
        leader_id: NodeId,
        leader_term: Term,
        prev_log_index: u64,
        prev_log_term: Term,
        entries: &[LogEntry],
        leader_commit: u64,
    ) -> AppendResult {
        let Some(peer) = self.peers.get_mut(&follower.index()) else {
            return AppendResult {
                term: Term(0),
                success: false,
                conflict_index: None,
            };
        };

        if leader_term.0 < peer.current_term.0 {
            return AppendResult {
                term: peer.current_term,
                success: false,
                conflict_index: None,
            };
        }

        // Valid leader heartbeat / append: adopt term and become follower.
        if leader_term.0 > peer.current_term.0
            || peer.role != RaftRole::Follower
            || peer.voted_for != Some(leader_id)
        {
            if leader_term.0 >= peer.current_term.0 {
                peer.current_term = leader_term;
                peer.role = RaftRole::Follower;
                peer.voted_for = Some(leader_id);
            }
        }

        // Log matching: reject if prev does not match.
        if prev_log_index > 0 {
            match peer.term_at(prev_log_index) {
                Some(t) if t == prev_log_term => {}
                Some(_) => {
                    // Conflict: truncate from prev_log_index and report.
                    let conflict = prev_log_index;
                    return AppendResult {
                        term: peer.current_term,
                        success: false,
                        conflict_index: Some(conflict),
                    };
                }
                None => {
                    return AppendResult {
                        term: peer.current_term,
                        success: false,
                        conflict_index: Some(peer.last_log_index().saturating_add(1).max(1)),
                    };
                }
            }
        }

        // Append new entries; truncate conflicts first.
        for e in entries {
            let idx = e.index;
            if let Some(existing) = peer.entry_at(idx) {
                if existing.term != e.term {
                    // Delete existing entry and all that follow.
                    peer.log.truncate((idx - 1) as usize);
                    peer.log.push(e.clone());
                }
                // else: identical, skip
            } else {
                // Must be contiguous.
                if e.index != peer.last_log_index() + 1 {
                    return AppendResult {
                        term: peer.current_term,
                        success: false,
                        conflict_index: Some(peer.last_log_index() + 1),
                    };
                }
                peer.log.push(e.clone());
            }
        }

        if leader_commit > peer.commit_index {
            let last = peer.last_log_index();
            peer.commit_index = leader_commit.min(last);
        }

        AppendResult {
            term: peer.current_term,
            success: true,
            conflict_index: None,
        }
    }

    /// Leader appends a command, replicates to online followers, advances commit.
    ///
    /// Returns commit evidence. The command is committed only when a majority
    /// of the configured voter set has matched the new index in the leader term.
    pub fn propose(
        &mut self,
        leader: NodeId,
        command: LogCommand,
        online: &[NodeId],
    ) -> Result<ProposeResult, ProposeError> {
        let (term, new_index, entry) = {
            let peer = self
                .peers
                .get_mut(&leader.index())
                .ok_or(ProposeError::NotLeader)?;
            if peer.role != RaftRole::Leader {
                return Err(ProposeError::NotLeader);
            }
            let term = peer.current_term;
            let new_index = peer.last_log_index() + 1;
            let entry = LogEntry {
                term,
                index: new_index,
                command,
            };
            peer.log.push(entry.clone());
            peer.match_index.insert(leader.index(), new_index);
            peer.next_index.insert(leader.index(), new_index + 1);
            (term, new_index, entry)
        };

        // Replicate to online followers.
        let mut acked = vec![leader];
        let followers: Vec<NodeId> = self
            .voters
            .iter()
            .copied()
            .filter(|v| *v != leader && online.iter().any(|n| n == v))
            .collect();

        for follower in followers {
            self.replicate_to(leader, follower)?;
            let match_idx = self
                .peers
                .get(&leader.index())
                .and_then(|p| p.match_index.get(&follower.index()).copied())
                .unwrap_or(0);
            if match_idx >= new_index {
                acked.push(follower);
            }
        }

        // Advance commit_index on leader (Raft §5.4.2).
        self.advance_commit(leader);
        let commit_index = self
            .peers
            .get(&leader.index())
            .map(|p| p.commit_index)
            .unwrap_or(0);

        let committed = commit_index >= new_index;

        // Push updated leader_commit to followers that already matched.
        if committed {
            for follower in self.voters.clone() {
                if follower == leader {
                    continue;
                }
                if !online.iter().any(|n| *n == follower) {
                    continue;
                }
                let _ = self.replicate_to(leader, follower);
            }
        }

        Ok(ProposeResult {
            entry,
            term,
            position: LogPosition(new_index),
            replica_acks: acked.len() as u32,
            acked_by: acked,
            committed,
            commit_index,
        })
    }

    /// Replicate from leader to one follower (backoff next_index on conflict).
    fn replicate_to(&mut self, leader: NodeId, follower: NodeId) -> Result<(), ProposeError> {
        // Loop until success or prev falls to 0 with empty send of remaining.
        for _ in 0..64 {
            let (term, prev_idx, prev_term, entries, leader_commit) = {
                let peer = self
                    .peers
                    .get(&leader.index())
                    .ok_or(ProposeError::NotLeader)?;
                if peer.role != RaftRole::Leader {
                    return Err(ProposeError::NotLeader);
                }
                let next = *peer.next_index.get(&follower.index()).unwrap_or(&1);
                let prev_idx = next.saturating_sub(1);
                let prev_term = peer.term_at(prev_idx).unwrap_or(Term(0));
                let entries: Vec<LogEntry> = peer
                    .log
                    .iter()
                    .filter(|e| e.index >= next)
                    .cloned()
                    .collect();
                (
                    peer.current_term,
                    prev_idx,
                    prev_term,
                    entries,
                    peer.commit_index,
                )
            };

            let res = self.append_entries(
                follower,
                leader,
                term,
                prev_idx,
                prev_term,
                &entries,
                leader_commit,
            );

            if res.term.0 > term.0 {
                // Step down leader.
                if let Some(p) = self.peers.get_mut(&leader.index()) {
                    p.become_follower(res.term);
                }
                return Err(ProposeError::SteppedDown(res.term));
            }

            if res.success {
                let match_idx = if entries.is_empty() {
                    prev_idx
                } else {
                    entries.last().map(|e| e.index).unwrap_or(prev_idx)
                };
                if let Some(p) = self.peers.get_mut(&leader.index()) {
                    p.match_index.insert(follower.index(), match_idx);
                    p.next_index.insert(follower.index(), match_idx + 1);
                }
                return Ok(());
            }

            // Back off next_index.
            if let Some(p) = self.peers.get_mut(&leader.index()) {
                let next = p.next_index.entry(follower.index()).or_insert(1);
                let hint = res.conflict_index.unwrap_or(next.saturating_sub(1));
                *next = hint.max(1).min(*next).saturating_sub(0);
                // Always decrease by at least 1 when conflict without useful hint.
                if *next > 1 {
                    *next -= 1;
                } else if entries.is_empty() && prev_idx == 0 {
                    // Cannot make progress.
                    return Ok(());
                } else {
                    *next = 1;
                }
            }
        }
        Ok(())
    }

    fn advance_commit(&mut self, leader: NodeId) {
        let Some(peer) = self.peers.get(&leader.index()) else {
            return;
        };
        if peer.role != RaftRole::Leader {
            return;
        }
        let term = peer.current_term;
        let last = peer.last_log_index();
        let mut new_commit = peer.commit_index;

        // Check from highest index down to commit_index+1.
        for n in (peer.commit_index + 1..=last).rev() {
            let entry_term = match peer.term_at(n) {
                Some(t) => t,
                None => continue,
            };
            if entry_term != term {
                continue; // only commit current-term entries by majority
            }
            let mut count = 0u32;
            for v in &self.voters {
                let m = peer.match_index.get(&v.index()).copied().unwrap_or(0);
                if m >= n {
                    count += 1;
                }
            }
            if count >= self.quorum() {
                new_commit = n;
                break;
            }
        }

        if new_commit > peer.commit_index {
            if let Some(p) = self.peers.get_mut(&leader.index()) {
                p.commit_index = new_commit;
            }
        }
    }

    /// Entries that still need applying on `node` (`last_applied < commit_index`).
    pub fn take_apply_batch(&mut self, node: NodeId) -> Vec<LogEntry> {
        let Some(peer) = self.peers.get_mut(&node.index()) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        while peer.last_applied < peer.commit_index {
            let next = peer.last_applied + 1;
            if let Some(e) = peer.entry_at(next).cloned() {
                out.push(e);
                peer.last_applied = next;
            } else {
                break;
            }
        }
        out
    }

    /// Force-sync follower commit from leader after propose (apply path helper).
    pub fn sync_follower_commit(&mut self, leader: NodeId, follower: NodeId) {
        let leader_commit = self
            .peers
            .get(&leader.index())
            .map(|p| p.commit_index)
            .unwrap_or(0);
        if let Some(f) = self.peers.get_mut(&follower.index()) {
            let last = f.last_log_index();
            if leader_commit > f.commit_index {
                f.commit_index = leader_commit.min(last);
            }
        }
    }

    /// Build commit evidence for a position from the leader's match_index map.
    pub fn commit_evidence(&self, leader: NodeId, position: LogPosition) -> CommitEvidence {
        let peer = self.peers.get(&leader.index());
        let term = peer
            .and_then(|p| p.term_at(position.0))
            .unwrap_or(Term(0));
        let mut acked_by = Vec::new();
        if let Some(p) = peer {
            for v in &self.voters {
                let m = p.match_index.get(&v.index()).copied().unwrap_or(0);
                if m >= position.0 {
                    acked_by.push(*v);
                }
            }
        }
        let committed = peer
            .map(|p| p.commit_index >= position.0)
            .unwrap_or(false);
        CommitEvidence {
            partition: self.partition,
            term,
            position,
            acked_by,
            committed,
        }
    }
}

/// Election failure reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectError {
    /// Candidate is not in the voter set.
    NotAVoter,
    /// Candidate node is offline.
    CandidateOffline,
    /// No voting nodes are online.
    NoOnlineVoters,
    /// Votes short of quorum.
    NoQuorum {
        /// Votes obtained (including self).
        votes: u32,
        /// Votes required.
        need: u32,
    },
    /// Observed a higher term during election.
    HigherTerm(Term),
}

/// Propose / replicate failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposeError {
    /// Caller is not the current leader.
    NotLeader,
    /// Leader stepped down after seeing a higher term.
    SteppedDown(Term),
}

/// Successful propose outcome.
#[derive(Debug, Clone)]
pub struct ProposeResult {
    /// Appended entry.
    pub entry: LogEntry,
    /// Leader term.
    pub term: Term,
    /// Log position of the entry.
    pub position: LogPosition,
    /// Number of replicas that matched this index (including leader).
    pub replica_acks: u32,
    /// Nodes that acknowledged the index.
    pub acked_by: Vec<NodeId>,
    /// Whether the entry is committed under Raft rules.
    pub committed: bool,
    /// Leader commit_index after the propose.
    pub commit_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_node_group() -> PartitionRaft {
        PartitionRaft::new(
            PartitionId::new(0),
            vec![NodeId::new(0), NodeId::new(1), NodeId::new(2)],
            PlacementEpoch(1),
        )
    }

    #[test]
    fn election_requires_majority_of_configured_voters() {
        let mut g = three_node_group();
        // Only one online → cannot get 2 votes.
        let err = g
            .elect(NodeId::new(0), &[NodeId::new(0)])
            .unwrap_err();
        assert!(matches!(err, ElectError::NoQuorum { votes: 1, need: 2 }));

        let (leader, term) = g
            .elect(NodeId::new(0), &[NodeId::new(0), NodeId::new(1)])
            .unwrap();
        assert_eq!(leader, NodeId::new(0));
        assert!(term.0 >= 1);
        assert_eq!(g.current_leader(), Some((NodeId::new(0), term)));
    }

    #[test]
    fn log_matching_rejects_conflicting_prev() {
        let mut g = three_node_group();
        let online = [NodeId::new(0), NodeId::new(1), NodeId::new(2)];
        let (leader, term) = g.ensure_leader(&online).unwrap();

        let r = g
            .propose(
                leader,
                LogCommand::Put {
                    subject: "a".into(),
                    value: b"1".to_vec(),
                },
                &online,
            )
            .unwrap();
        assert!(r.committed);

        // Craft a bogus append with wrong prev term.
        let res = g.append_entries(
            NodeId::new(1),
            leader,
            term,
            1,
            Term(999), // wrong
            &[],
            0,
        );
        assert!(!res.success);
    }

    #[test]
    fn cannot_commit_without_quorum_replication() {
        let mut g = three_node_group();
        // Elect with two nodes, then propose with only leader online.
        let (leader, _) = g
            .elect(NodeId::new(0), &[NodeId::new(0), NodeId::new(1)])
            .unwrap();
        // Mark only leader online for propose.
        let r = g
            .propose(
                leader,
                LogCommand::Put {
                    subject: "x".into(),
                    value: b"y".to_vec(),
                },
                &[NodeId::new(0)],
            )
            .unwrap();
        assert!(!r.committed);
        assert_eq!(r.replica_acks, 1);
    }

    #[test]
    fn commit_with_majority() {
        let mut g = three_node_group();
        let online = [NodeId::new(0), NodeId::new(1), NodeId::new(2)];
        let (leader, _) = g.ensure_leader(&online).unwrap();
        let r = g
            .propose(
                leader,
                LogCommand::Delete {
                    subject: "z".into(),
                },
                &online,
            )
            .unwrap();
        assert!(r.committed);
        assert!(r.replica_acks >= 2);
        assert_eq!(r.position, LogPosition(1));
    }

    #[test]
    fn more_up_to_date_log_wins_election() {
        let mut g = three_node_group();
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

        // Node 2 has empty log; node 0/1 have the entry. Take node 0 offline;
        // node 1 should win over node 2.
        if let Some(p) = g.peer_mut(NodeId::new(0)) {
            p.role = RaftRole::Follower;
        }
        let (leader, _) = g
            .ensure_leader(&[NodeId::new(1), NodeId::new(2)])
            .unwrap();
        assert_eq!(leader, NodeId::new(1));
    }
}
