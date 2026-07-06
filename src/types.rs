use serde::{Deserialize, Serialize};

/// Identifies a replica (validator) in the consensus group.
pub type NodeId = u64;

/// A monotonically increasing view number. Each view has a designated leader.
pub type View = u64;

/// A sequence number for client requests / committed commands.
pub type Sequence = u64;

/// A cryptographic hash digest.
pub type Digest = [u8; 32];

/// The set of replicas that participate in consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSet {
    /// All replica ids in canonical order.
    pub nodes: Vec<NodeId>,
    /// The minimum number of honest replicas we assume: `n = 3f + 1`.
    pub f: usize,
}

impl ReplicaSet {
    /// Total number of replicas `n`.
    pub fn n(&self) -> usize {
        self.nodes.len()
    }

    /// The quorum size needed for a certificate: `2f + 1`.
    pub fn quorum_size(&self) -> usize {
        2 * self.f + 1
    }

    /// Returns the leader for the given view (round-robin).
    pub fn leader_of(&self, view: View) -> NodeId {
        self.nodes[(view as usize) % self.nodes.len()]
    }

    /// Returns true iff `id` is the leader of `view`.
    pub fn is_leader_of(&self, id: NodeId, view: View) -> bool {
        self.leader_of(view) == id
    }
}
