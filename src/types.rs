use serde::{Deserialize, Serialize};

/// Delta, the number of milliseconds a replica waits for a decision before timing out and moving 
/// to the next view. This comes from the assumption of all BFT algorithm in the partial synchronous
/// model, where the network is assumed to be synchronous after some unknown global stabilization 
/// time (GST).
/// The raw paper use a fix 3*Delta timeout, it assumes that slow client can always catch up within
/// 3 times of delta time window, but in reality, we consider an exponential backoff timeout, 
/// where the timeout is increased by a factor of 2 for the continuously accumulating number of 
/// timeout view changes, to avoid unnecessary view changes due to slow clients.
pub const DELTA: u64 = 500; // 500ms

/// Dummy block hash. On timeout, each node vote for a dummy block to form a timeout certificate.
pub const DUMMY_BLOCK_HASH: Digest = [0u8; 32];

/// Identifies a replica (validator) in the consensus group.
pub type NodeId = usize;

/// A monotonically increasing height number. Each height has a designated leader.
pub type Height = u64;

/// A cryptographic hash digest.
pub type Digest = [u8; 32];

/// The set of validators that participate in consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeSet {
    /// All validator ids in canonical order.
    pub nodes: Vec<NodeId>,
    /// The minimum number of honest validators we assume: `n = 3f + 1`.
    pub f: usize,
}

impl CommitteeSet {
    /// Total number of validators `n`.
    pub fn n(&self) -> usize {
        self.nodes.len()
    }

    /// The quorum size needed for a certificate: `2f + 1`.
    pub fn quorum_size(&self) -> usize {
        2 * self.f + 1
    }

    /// Returns the leader for the given view (round-robin).
    pub fn leader_of(&self, view: Height) -> NodeId {
        self.nodes[(view as usize) % self.nodes.len()]
    }

    /// Returns true iff `id` is the leader of `view`.
    pub fn is_leader_of(&self, id: NodeId, view: Height) -> bool {
        self.leader_of(view) == id
    }
}
