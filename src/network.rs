use crate::message::Message;
use crate::types::NodeId;

/// The consensus engine's only interaction point with the outside world for message
/// delivery. Implementations may use TCP, gRPC, QUIC, libp2p, etc.
///
/// The engine calls [`Network::broadcast`] and receives messages via a push-based
/// callback ([`MessageHandler`]).
pub trait Network<P>: Send + Sync
where
    P: Send + Sync,
{
    /// Broadcast a protocol message to **all** replicas (including self).
    fn broadcast(&mut self, msg: Message<P>) -> Result<(), crate::error::ConsensusError>;

    /// Send a message to a specific replica.
    fn send_to(&mut self, to: NodeId, msg: Message<P>) -> Result<(), crate::error::ConsensusError>;
}

/// Consumed by the host to push an incoming message into the consensus engine.
pub trait MessageHandler<P>
where
    P: Send + Sync,
{
    /// Called by the networking layer when a protocol message arrives for this replica.
    fn handle_message(
        &mut self,
        from: NodeId,
        msg: Message<P>,
    ) -> Result<(), crate::error::ConsensusError>;
}
