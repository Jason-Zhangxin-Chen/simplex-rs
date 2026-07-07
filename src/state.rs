use crate::types::{Digest};

/// Abstraction over the replicated state machine (blockchain, KV store, etc.).
///
/// The consensus engine commits commands in total order via [`StateMachine::apply`]
/// and the host queries the current state digest via [`StateMachine::digest`].
pub trait StateMachine<P>: Send + Sync {
    /// Validate a command **without** applying it. Used during proposal verification.
    fn validate(&self, command: &P) -> Result<(), crate::error::ConsensusError>;

    /// Apply a batch of commands in the given total order.
    ///
    /// Returns a digest of the new state after applying this batch.
    fn apply(
        &mut self,
        batch: &[P],
    ) -> Result<Digest, crate::error::ConsensusError>;

    /// Current state digest.
    fn digest(&self) -> Digest;
    
    /// todo: add a subscriber that the clients can subscribe the chain head event to update the
    /// latest view from consensus engine.
    /// 
    /// todo: add a subscriber that the clients can subscribe the new mined block on top of 
    /// chain head. It is being used to propose a new proposal.
}
