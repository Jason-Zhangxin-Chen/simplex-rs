use crate::types::Digest;

/// Trait that the host system must implement or provide so the consensus engine can
/// verify signed messages.
///
/// The consensus core never touches private keys — it only verifies and hashes.
pub trait CryptoProvider: Send + Sync {
    /// Verify that `signature` is a valid signature over `message` under `node`'s public key.
    fn verify(
        &self,
        node: crate::types::NodeId,
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), crate::error::ConsensusError>;

    /// Hash `data` into a 32-byte digest.
    fn hash(&self, data: &[u8]) -> Digest;

    /// Combine two or more digests (e.g. for a Merkle-like structure).
    fn hash_pair(&self, a: &Digest, b: &Digest) -> Digest {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(a);
        buf[32..].copy_from_slice(b);
        self.hash(&buf)
    }
}

/// A signer is needed by the node that owns this replica to produce signatures for
/// outgoing messages. Downstream crates wire this to the actual key store.
pub trait Signer: Send + Sync {
    /// Sign `message` and return the signature bytes.
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, crate::error::ConsensusError>;

    /// The `NodeId` of the signer.
    fn node_id(&self) -> crate::types::NodeId;
}
