//! Consistency and read modes (CLUSTER_SPEC §9).

use serde::{Deserialize, Serialize};

/// Store / namespace write consistency mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConsistencyMode {
    /// One consensus-authorized leader orders writes; quorum commits (default).
    #[default]
    PartitionLinearizable,
    /// Multi-side append for mergeable immutable events; not linearizable.
    ConvergentAppend,
}

impl ConsistencyMode {
    /// Stable name for receipts and metadata.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PartitionLinearizable => "partition-linearizable",
            Self::ConvergentAppend => "convergent-append",
        }
    }

    /// Parse a stable name.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "partition-linearizable" => Some(Self::PartitionLinearizable),
            "convergent-append" => Some(Self::ConvergentAppend),
            _ => None,
        }
    }
}

/// How a read interprets replica state (CLUSTER_SPEC §9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ReadMode {
    /// Proven current under partition consensus, or fail when evidence missing.
    #[default]
    Linearizable,
    /// Best verified reachable state with exact coverage (no absence proof).
    Available,
    /// All relevant verified physical evidence; recovery tools default.
    Salvage,
}

impl ReadMode {
    /// Stable name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linearizable => "linearizable",
            Self::Available => "available",
            Self::Salvage => "salvage",
        }
    }
}

/// Whether a physically surviving frame is known committed (CLUSTER_SPEC §10.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommitStatus {
    /// Quorum / consensus evidence proves commitment.
    Committed,
    /// Accepted by at least one replica; commitment unproven.
    Prepared,
    /// Verified evidence cannot belong to one valid committed log.
    Conflicting,
    /// Required consensus evidence is missing.
    UnknownCommit,
}

impl CommitStatus {
    /// Stable name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Prepared => "prepared",
            Self::Conflicting => "conflicting",
            Self::UnknownCommit => "unknown-commit",
        }
    }
}

/// Named deployment profile (CLUSTER_SPEC §23).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentProfile {
    /// One node, one replica; no fault tolerance.
    Development,
    /// Three voting storage nodes; quorum durable ack.
    DependableLocal,
}

impl DeploymentProfile {
    /// Stable name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::DependableLocal => "dependable-local",
        }
    }

    /// Default voting replica count for the profile.
    pub fn default_node_count(self) -> u32 {
        match self {
            Self::Development => 1,
            Self::DependableLocal => 3,
        }
    }

    /// Write quorum: floor(N/2)+1.
    pub fn write_quorum(self) -> u32 {
        let n = self.default_node_count();
        n / 2 + 1
    }
}
