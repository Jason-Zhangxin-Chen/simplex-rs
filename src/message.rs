use serde::{Deserialize, Serialize};

use crate::types::{Digest, NodeId, Height};

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

    /// Replicas vote for a finalize.
    Finalize(Finalize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal<P> {
    /// The sequence number (height) this proposal targets.
    pub height: Height,
    /// Digest of the parent block / previous proposal.
    pub parent: Digest,
    /// The batch of blocks to be proposed.
    pub payload: Vec<P>,
    /// A justification proving this proposal is safe (e.g. a quorum certificate from the
    /// previous view, or a timeout-certificate that triggered the view change).
    pub justification: Justification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// The sequence number (height) this vote targets.
    pub height: Height,
    /// Digest of the proposal being voted for.
    pub digest: Digest,
    /// The replica that cast this vote.
    pub voter: NodeId,
    /// A cryptographic signature over the vote.
    pub signature: SignatureBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finalize {
    /// The sequence number (height) this vote targets.
    pub height: Height,
    /// The replica that cast this vote.
    pub voter: NodeId,
    /// A cryptographic signature over the vote.
    pub signature: SignatureBytes,
}

/// A justification that accompanies a proposal so other replicas can verify its safety.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Justification {
    /// Quorum Certificate: a set of `2f+1` votes for a previous proposal.
    QuorumCertificate(QuorumCert),
    /// Timeout Certificate: `2f+1` timeout messages proving the height change is legitimate.
    TimeoutCertificate(TimeoutCert),
    /// Genesis / empty justification for the very first proposal.
    Genesis,
}

/// A set of `2f+1` votes that certifies a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCert {
    pub height: Height,
    pub digest: Digest,
    pub signatures: Vec<(NodeId, SignatureBytes)>,
}

/// A set of `2f+1` timeout messages that justifies advancing to a new height.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutCert {
    pub next_view: Height,
    pub signatures: Vec<(NodeId, SignatureBytes)>,
}

/// Opaque signature bytes. Real implementations will use e.g. ed25519 (64 bytes) or
/// BLS (48/96 bytes).
pub type SignatureBytes = Vec<u8>;
