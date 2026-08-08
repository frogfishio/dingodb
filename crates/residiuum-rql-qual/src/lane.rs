//! Comparison lanes — frozen by Q0 (`RQL_Q0_LANES_EXCLUSIONS.md`).
//!
//! Never score embedded Residiuum against MongoDB TCP as one contest.

use serde::{Deserialize, Serialize};

/// Gate-1 competitive lane id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneId {
    /// Lane E — Residiuum embedded vs Couchbase Lite embedded.
    Embedded,
    /// Lane S — Residiuum server (loopback) vs local MongoDB.
    LocalClientServer,
}

impl LaneId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::LocalClientServer => "local_client_server",
        }
    }
}

/// Engine under test or comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineId {
    ResidiuumEmbedded,
    ResidiuumServer,
    MongoLocal,
    CouchbaseLiteEmbedded,
}

impl EngineId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ResidiuumEmbedded => "residiuum_embedded",
            Self::ResidiuumServer => "residiuum_server",
            Self::MongoLocal => "mongo_local",
            Self::CouchbaseLiteEmbedded => "cbl_embedded",
        }
    }

    /// Lane this engine may participate in for Gate-1 cells.
    pub fn primary_lane(self) -> LaneId {
        match self {
            Self::ResidiuumEmbedded | Self::CouchbaseLiteEmbedded => LaneId::Embedded,
            Self::ResidiuumServer | Self::MongoLocal => LaneId::LocalClientServer,
        }
    }
}

/// One comparative pairing inside a single lane (side A vs side B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanePairing {
    pub lane: LaneId,
    pub side_a: EngineId,
    pub side_b: EngineId,
}

impl LanePairing {
    pub const EMBEDDED: Self = Self {
        lane: LaneId::Embedded,
        side_a: EngineId::ResidiuumEmbedded,
        side_b: EngineId::CouchbaseLiteEmbedded,
    };

    pub const LOCAL_CS: Self = Self {
        lane: LaneId::LocalClientServer,
        side_a: EngineId::ResidiuumServer,
        side_b: EngineId::MongoLocal,
    };

    /// Reject cross-lane pairings (architecture invariant).
    pub fn validate(self) -> Result<(), String> {
        if self.side_a.primary_lane() != self.lane {
            return Err(format!(
                "side_a {} not eligible for lane {}",
                self.side_a.as_str(),
                self.lane.as_str()
            ));
        }
        if self.side_b.primary_lane() != self.lane {
            return Err(format!(
                "side_b {} not eligible for lane {}",
                self.side_b.as_str(),
                self.lane.as_str()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_pairings_validate() {
        LanePairing::EMBEDDED.validate().unwrap();
        LanePairing::LOCAL_CS.validate().unwrap();
    }

    #[test]
    fn cross_lane_pairing_rejected() {
        let bad = LanePairing {
            lane: LaneId::Embedded,
            side_a: EngineId::ResidiuumEmbedded,
            side_b: EngineId::MongoLocal,
        };
        assert!(bad.validate().is_err());
    }
}
