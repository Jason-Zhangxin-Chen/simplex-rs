use serde::{Deserialize, Serialize};

use crate::types::{Digest, NodeId, Sequence, View};

/// Enumeration of all simplex protocol message types.
///
/// The generic parameter `P` is the **payload** — the opaque command / transaction blob
/// that the consensus engine does not interpret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message<P> {
    /// Leader proposes a batch of commands in a given view.
    Proposal(Proposal<P>),

    /// Replicas vote for a proposal they accept.
    Vote(Vote),

    /// Timeout / view-change message: a replica wishes to advance to a new view.
    Timeout(Timeout),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal<P> {
    /// The view this proposal belongs to.
    pub view: View,
    /// The sequence number (height) this proposal targets.
    pub sequence: Sequence,
    /// Digest of the parent block / previous proposal.
    pub parent: Digest,
    /// The batch of commands.
    pub payload: Vec<P>,
    /// A justification proving this proposal is safe (e.g. a quorum certificate from the
    /// previous view, or a timeout-certificate that triggered the view change).
    pub justification: Justification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub view: View,
    pub sequence: Sequence,
    /// Digest of the proposal being voted for.
    pub digest: Digest,
    /// The replica that cast this vote.
    pub voter: NodeId,
    /// A cryptographic signature over the vote.
    pub signature: SignatureBytes,
}

/// A timeout signals that a replica has not received a timely proposal and wishes to
/// move to `next_view`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeout {
    /// The view in which the timeout occurred.
    pub current_view: View,
    /// The view the replica wants to advance to.
    pub next_view: View,
    /// The highest `(view, sequence)` the replica has previously voted for (lock).
    pub high_vote: Option<(View, Sequence)>,
    /// The replica that issued the timeout.
    pub sender: NodeId,
    /// A cryptographic signature over the timeout.
    pub signature: SignatureBytes,
}

/// A justification that accompanies a proposal so other replicas can verify its safety.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Justification {
    /// Quorum Certificate: a set of `2f+1` votes for a previous proposal.
    QuorumCertificate(QuorumCert),
    /// Timeout Certificate: `2f+1` timeout messages proving the view change is legitimate.
    TimeoutCertificate(TimeoutCert),
    /// Genesis / empty justification for the very first proposal.
    Genesis,
}

/// A set of `2f+1` votes that certifies a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCert {
    pub view: View,
    pub sequence: Sequence,
    pub digest: Digest,
    pub signatures: Vec<(NodeId, SignatureBytes)>,
}

/// A set of `2f+1` timeout messages that justifies advancing to a new view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutCert {
    pub next_view: View,
    pub signatures: Vec<(NodeId, SignatureBytes)>,
}

/// Opaque signature bytes. Real implementations will use e.g. ed25519 (64 bytes) or
/// BLS (48/96 bytes).
pub type SignatureBytes = Vec<u8>;
