//! # simplex-rs
//!
//! A Rust implementation of the **Simplex BFT consensus** protocol.
//!
//! This crate provides the core consensus algorithm while abstracting away the
//! underlying blockchain state, network transport, and cryptographic primitives
//! behind traits. This way the protocol can be reused across different systems.
//!
//! ## References
//!
//! - [Simplex Consensus (original)](https://eprint.iacr.org/2023/463.pdf)
//! - [Improved Simplex](https://eprint.iacr.org/2023/1916.pdf)
//!
//! ## Architecture
//!
//! | Trait / Type                          | Role                                      |
//! |---------------------------------------|-------------------------------------------|
//! | [`StateMachine`](state::StateMachine) | The replicated application / ledger.      |
//! | [`Network`](network::Network)         | Reliable broadcast / point-to-point send. |
//! | [`CryptoProvider`](crypto::CryptoProvider) | Hash + signature verification.       |
//! | [`Signer`](crypto::Signer)            | Produces signatures for this replica.     |
//! | [`Simplex`](protocol::Simplex)        | The consensus engine itself.              |

pub mod config;
pub mod crypto;
pub mod error;
pub mod message;
pub mod network;
pub mod protocol;
pub mod state;
pub mod types;

// Convenience re-exports
pub use config::Config;
pub use crypto::{CryptoProvider, Signer};
pub use error::ConsensusError;
pub use network::Network;
pub use protocol::Simplex;
pub use state::StateMachine;
pub use types::{Digest, NodeId, CommitteeSet, Height};
