//! Server-side network Raft runtime (DEF-036).
//!
//! Holds per-partition [`NetworkRaftNode`] state for one process and dispatches
//! inbound control-plane RPCs. Outbound RPCs use [`TcpRaftTransport`] over the
//! framed application protocol (`raft_*` ops).

use crate::error::Error;
use crate::remote::{ConnectOptions, RemoteClient, RpcRequest};
use dingo_cluster::raft_rpc::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    NetworkRaftNode, RaftTransport, ReadIndexRequest, ReadIndexResponse, RequestVoteRequest,
    RequestVoteResponse,
};
use dingo_cluster::{
    ClusterId, ClusterMeta, NodeId, PartitionDirectory, PartitionId, PlacementEpoch, RaftPeerStore,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Feature token advertised when a server speaks network Raft RPC.
pub const FEATURE_RAFT_RPC_V1: &str = "raft-rpc-v1";

/// Re-export profile for capability matrices.
pub use dingo_cluster::RAFT_RPC_PROFILE as SDK_RAFT_RPC_PROFILE;

/// Per-process Raft control-plane state for one cluster node.
#[derive(Debug)]
pub struct RaftServerState {
    /// Cluster identity.
    pub cluster_id: ClusterId,
    /// Dense node index of this process.
    pub local: NodeId,
    /// Optional shared auth token for peer RPCs.
    pub peer_token: Option<String>,
    /// partition → local network node.
    nodes: HashMap<u32, NetworkRaftNode>,
    /// Endpoint map node_index → host:port (routing hints only).
    endpoints: HashMap<u32, String>,
}

impl RaftServerState {
    /// Build from a cluster root for `node_index`, attaching durable peer stores.
    pub fn open(
        cluster_root: &Path,
        node_index: u32,
        peer_token: Option<String>,
    ) -> Result<Self, Error> {
        let meta = ClusterMeta::load(cluster_root)
            .map_err(|e| Error::Internal(format!("cluster meta: {e}")))?;
        if node_index >= meta.node_count {
            return Err(Error::ValidationMsg(format!(
                "node {node_index} out of range ({} nodes)",
                meta.node_count
            )));
        }
        let cluster_id = ClusterId::from_hex(&meta.cluster_id).ok_or_else(|| {
            Error::Internal(format!("invalid cluster_id hex in meta: {}", meta.cluster_id))
        })?;
        let directory = PartitionDirectory::load(cluster_root)
            .map_err(|e| Error::Internal(format!("placement: {e}")))?
            .ok_or_else(|| Error::Internal("missing placement.json".into()))?;
        let endpoints = dingo_cluster::load_endpoints(cluster_root).unwrap_or_default();
        let local = NodeId::new(node_index);
        let mut nodes = HashMap::new();
        for a in &directory.assignments {
            if !a.replicas.iter().any(|r| *r == local) {
                continue;
            }
            let store = RaftPeerStore::open(cluster_root, local, a.partition)
                .map_err(|e| Error::Internal(format!("raft store: {e}")))?;
            let mut node = NetworkRaftNode::new(
                cluster_id,
                a.partition,
                local,
                a.replicas.clone(),
                a.placement_epoch,
            );
            node.attach_store(store)
                .map_err(|e| Error::Internal(format!("raft attach: {e}")))?;
            nodes.insert(a.partition.0, node);
        }
        Ok(Self {
            cluster_id,
            local,
            peer_token,
            nodes,
            endpoints,
        })
    }

    /// In-memory (no disk) state for tests — one partition, given voters.
    pub fn for_test(
        cluster_id: ClusterId,
        local: NodeId,
        partition: PartitionId,
        voters: Vec<NodeId>,
        placement_epoch: PlacementEpoch,
        endpoints: HashMap<u32, String>,
        peer_token: Option<String>,
    ) -> Self {
        let node = NetworkRaftNode::new(cluster_id, partition, local, voters, placement_epoch);
        let mut nodes = HashMap::new();
        nodes.insert(partition.0, node);
        Self {
            cluster_id,
            local,
            peer_token,
            nodes,
            endpoints,
        }
    }

    /// Update endpoint routing hints (not write authority).
    pub fn set_endpoints(&mut self, endpoints: HashMap<u32, String>) {
        self.endpoints = endpoints;
    }

    /// Refresh endpoints from cluster root.
    pub fn reload_endpoints(&mut self, cluster_root: &Path) {
        if let Ok(eps) = dingo_cluster::load_endpoints(cluster_root) {
            self.endpoints = eps;
        }
    }

    /// Borrow a partition node.
    pub fn node_mut(&mut self, partition: u32) -> Option<&mut NetworkRaftNode> {
        self.nodes.get_mut(&partition)
    }

    /// Build a transport that dials peers by endpoint map.
    pub fn transport(&self) -> TcpRaftTransport {
        TcpRaftTransport {
            endpoints: self.endpoints.clone(),
            token: self.peer_token.clone(),
        }
    }

    /// Online peers we have an endpoint for (routing hint; not authority).
    pub fn online_hint(&self) -> Vec<NodeId> {
        let mut v: Vec<NodeId> = self
            .endpoints
            .keys()
            .copied()
            .map(NodeId::new)
            .collect();
        if !v.iter().any(|n| *n == self.local) {
            v.push(self.local);
        }
        v.sort_by_key(|n| n.index());
        v
    }

    /// Campaign for leadership on `partition`.
    pub fn campaign(&mut self, partition: u32) -> Result<(NodeId, dingo_cluster::Term), Error> {
        let online = self.online_hint();
        let transport = self.transport();
        let node = self
            .nodes
            .get_mut(&partition)
            .ok_or_else(|| Error::ValidationMsg(format!("no raft state for p{partition}")))?;
        node.campaign(&transport, &online)
            .map_err(|e| Error::Internal(format!("campaign: {e:?}")))
    }

    /// Dispatch an inbound raft RPC by op name and JSON body.
    pub fn dispatch_json(&mut self, op: &str, body: &JsonValue) -> Result<JsonValue, Error> {
        match op {
            "raft_request_vote" => {
                let req: RequestVoteRequest = serde_json::from_value(body.clone())
                    .map_err(|e| Error::ProtocolViolation(format!("request_vote: {e}")))?;
                let node = self
                    .nodes
                    .get_mut(&req.partition)
                    .ok_or_else(|| Error::ValidationMsg("unknown partition".into()))?;
                let resp = node
                    .handle_request_vote(&req)
                    .map_err(raft_err)?;
                Ok(serde_json::to_value(resp).expect("serialize"))
            }
            "raft_append_entries" => {
                let req: AppendEntriesRequest = serde_json::from_value(body.clone())
                    .map_err(|e| Error::ProtocolViolation(format!("append_entries: {e}")))?;
                let node = self
                    .nodes
                    .get_mut(&req.partition)
                    .ok_or_else(|| Error::ValidationMsg("unknown partition".into()))?;
                let resp = node
                    .handle_append_entries(&req)
                    .map_err(raft_err)?;
                Ok(serde_json::to_value(resp).expect("serialize"))
            }
            "raft_install_snapshot" => {
                let req: InstallSnapshotRequest = serde_json::from_value(body.clone())
                    .map_err(|e| Error::ProtocolViolation(format!("install_snapshot: {e}")))?;
                let node = self
                    .nodes
                    .get_mut(&req.partition)
                    .ok_or_else(|| Error::ValidationMsg("unknown partition".into()))?;
                let resp = node
                    .handle_install_snapshot(&req)
                    .map_err(raft_err)?;
                Ok(serde_json::to_value(resp).expect("serialize"))
            }
            "raft_read_index" => {
                let req: ReadIndexRequest = serde_json::from_value(body.clone())
                    .map_err(|e| Error::ProtocolViolation(format!("read_index: {e}")))?;
                let node = self
                    .nodes
                    .get_mut(&req.partition)
                    .ok_or_else(|| Error::ValidationMsg("unknown partition".into()))?;
                let resp = node.handle_read_index(&req).map_err(raft_err)?;
                Ok(serde_json::to_value(resp).expect("serialize"))
            }
            _ => Err(Error::ValidationMsg(format!("unknown raft op {op}"))),
        }
    }
}

fn raft_err(e: dingo_cluster::RaftRpcError) -> Error {
    match e {
        dingo_cluster::RaftRpcError::Unauthorized(s) => Error::AuthenticationFailed(s),
        dingo_cluster::RaftRpcError::Fenced(s) => Error::ConsistencyViolation(s),
        dingo_cluster::RaftRpcError::Protocol(s) => Error::ProtocolViolation(s),
        other => Error::Internal(other.to_string()),
    }
}

/// Shared handle placed on [`crate::remote::ServeOptions`].
pub type SharedRaftState = Arc<Mutex<RaftServerState>>;

/// TCP transport: dials peer endpoints and issues `raft_*` application RPCs.
#[derive(Debug, Clone)]
pub struct TcpRaftTransport {
    /// Routing hints only (node_index → host:port).
    pub endpoints: HashMap<u32, String>,
    /// Shared auth token for peer calls.
    pub token: Option<String>,
}

impl TcpRaftTransport {
    fn call_json(&self, to: NodeId, op: &str, body: &JsonValue) -> Result<JsonValue, Error> {
        let addr = self
            .endpoints
            .get(&to.index())
            .ok_or_else(|| {
                Error::Internal(format!(
                    "no endpoint for {to} (routing hint missing; not write authority)"
                ))
            })?
            .clone();
        let mut opts = ConnectOptions::new();
        if let Some(t) = &self.token {
            opts = opts.auth_token(t.clone());
        }
        // endpoint label is diagnostic only; addr is the dial target.
        let mut client = RemoteClient::connect_with(&addr, addr.clone(), opts)?;
        let req = RpcRequest {
            id: 1,
            op: op.into(),
            collection: None,
            key: None,
            json: Some(body.clone()),
            bytes_b64: None,
            limit: None,
            durability: None,
            token: self.token.clone(),
            fields: None,
            force_scan: None,
            order_field: None,
            order_dir: None,
            max_docs_scanned: None,
            max_bytes_scanned: None,
            max_result_bytes: None,
            operation_id: None,
            confirm: None,
        };
        let resp = client.call_rpc(req)?;
        if !resp.ok {
            return Err(Error::Internal(
                resp.error
                    .unwrap_or_else(|| format!("raft rpc {op} failed")),
            ));
        }
        resp.value
            .ok_or_else(|| Error::ProtocolViolation(format!("{op} missing value payload")))
    }
}

impl RaftTransport for TcpRaftTransport {
    fn request_vote(
        &self,
        to: NodeId,
        req: &RequestVoteRequest,
    ) -> Result<RequestVoteResponse, dingo_cluster::RaftRpcError> {
        let body = serde_json::to_value(req).map_err(|e| {
            dingo_cluster::RaftRpcError::Protocol(format!("encode request_vote: {e}"))
        })?;
        let v = self
            .call_json(to, "raft_request_vote", &body)
            .map_err(|e| dingo_cluster::RaftRpcError::Unavailable(e.to_string()))?;
        serde_json::from_value(v)
            .map_err(|e| dingo_cluster::RaftRpcError::Protocol(format!("decode: {e}")))
    }

    fn append_entries(
        &self,
        to: NodeId,
        req: &AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse, dingo_cluster::RaftRpcError> {
        let body = serde_json::to_value(req).map_err(|e| {
            dingo_cluster::RaftRpcError::Protocol(format!("encode append_entries: {e}"))
        })?;
        let v = self
            .call_json(to, "raft_append_entries", &body)
            .map_err(|e| dingo_cluster::RaftRpcError::Unavailable(e.to_string()))?;
        serde_json::from_value(v)
            .map_err(|e| dingo_cluster::RaftRpcError::Protocol(format!("decode: {e}")))
    }

    fn install_snapshot(
        &self,
        to: NodeId,
        req: &InstallSnapshotRequest,
    ) -> Result<InstallSnapshotResponse, dingo_cluster::RaftRpcError> {
        let body = serde_json::to_value(req).map_err(|e| {
            dingo_cluster::RaftRpcError::Protocol(format!("encode install_snapshot: {e}"))
        })?;
        let v = self
            .call_json(to, "raft_install_snapshot", &body)
            .map_err(|e| dingo_cluster::RaftRpcError::Unavailable(e.to_string()))?;
        serde_json::from_value(v)
            .map_err(|e| dingo_cluster::RaftRpcError::Protocol(format!("decode: {e}")))
    }

    fn read_index(
        &self,
        to: NodeId,
        req: &ReadIndexRequest,
    ) -> Result<ReadIndexResponse, dingo_cluster::RaftRpcError> {
        let body = serde_json::to_value(req)
            .map_err(|e| dingo_cluster::RaftRpcError::Protocol(format!("encode read_index: {e}")))?;
        let v = self
            .call_json(to, "raft_read_index", &body)
            .map_err(|e| dingo_cluster::RaftRpcError::Unavailable(e.to_string()))?;
        serde_json::from_value(v)
            .map_err(|e| dingo_cluster::RaftRpcError::Protocol(format!("decode: {e}")))
    }
}

/// Helper: open raft state and wrap in a shared mutex.
pub fn shared_raft_state(
    cluster_root: impl AsRef<Path>,
    node_index: u32,
    peer_token: Option<String>,
) -> Result<SharedRaftState, Error> {
    let state = RaftServerState::open(cluster_root.as_ref(), node_index, peer_token)?;
    Ok(Arc::new(Mutex::new(state)))
}


