use std::time::Duration;

/// Tunable parameters for the Simplex protocol.
#[derive(Debug, Clone)]
pub struct Config {
    /// The replica that owns this configuration.
    pub own_id: super::types::NodeId,

    /// The full replica set.
    pub replica_set: super::types::CommitteeSet,

    /// Base timeout for a view change. Multiplied by `view` for exponential back-off
    /// (or tune via `view_timeout_fn`).
    pub view_timeout_base: Duration,

    /// How many views ahead we allow votes/locks to carry over before requiring a fresh proposal.
    pub max_view_gap: u64,

    /// Batch size limit (bytes) for a single proposal payload.
    pub max_payload_bytes: usize,
}

impl Config {
    /// Calculate the timeout for a given view using exponential back-off capped at
    /// `base * 2^(view)` up to a reasonable maximum.
    pub fn view_timeout(&self, view: super::types::Height) -> Duration {
        let exp = view.min(10); // cap the exponent
        self.view_timeout_base * 2u32.pow(exp as u32)
    }
}
