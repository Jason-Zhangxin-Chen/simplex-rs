# simplex-rs

A Rust implementation of the **Simplex BFT consensus** protocol.

For details please refer to the papers:

- [Simplex Consensus (original)](https://eprint.iacr.org/2023/463.pdf)
- [Improved Simplex](https://eprint.iacr.org/2023/1916.pdf)

The goal of this crate is to abstract the underlying blockchain and network layer, focusing on the internal protocol of the algorithm so that it can be reused in different systems.

## Architecture

The consensus engine (`Simplex`) is generic over four pluggable traits. Downstream crates wire in concrete implementations:

| Trait | Role | You implement |
|---|---|---|
| `StateMachine<P>` | The replicated application / ledger | Validate and apply commands, return state digest |
| `Network<P>` | Message transport | broadcast / point-to-point send (TCP, gRPC, QUIC, …) |
| `CryptoProvider` | Hash + signature verification | `verify()`, `hash()` |
| `Signer` | Produce signatures for this replica | `sign()`, `node_id()` |

The payload type `P` (transaction / command blob) is opaque to the engine.

## Crate structure

```
src/
├── lib.rs          # Crate root — module declarations + re-exports
├── types.rs        # NodeId, View, Sequence, Digest, ReplicaSet
├── config.rs       # Tunable protocol parameters (timeouts, batch limits, …)
├── error.rs        # ConsensusError enum
├── message.rs      # Wire types: Proposal<P>, Vote, Timeout, QuorumCert, TimeoutCert
├── crypto.rs       # Traits: CryptoProvider, Signer
├── network.rs      # Traits: Network<P>, MessageHandler<P>
├── state.rs        # Trait: StateMachine<P>
└── protocol.rs     # Simplex<P, N, S, C, K> — the consensus engine
```

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
simplex-rs = { git = "…" }
```

Implement the four traits for your system, then drive the engine:

```rust
use simplex_rs::{Config, ReplicaSet, Simplex};

// 1. Build config
let config = Config {
    own_id: 0,
    replica_set: ReplicaSet {
        nodes: vec![0, 1, 2, 3],  // 4 nodes, f = 1
        f: 1,
    },
    view_timeout_base: Duration::from_secs(1),
    max_view_gap: 10,
    max_payload_bytes: 1_048_576,
};

// 2. Wire up your implementations
let engine = Simplex::new(config, my_network, my_state, my_crypto, my_signer);

// 3. Feed incoming messages
engine.on_message(from, msg)?;

// 4. Bootstrap or advance views
engine.start_view(0)?;
```

## Status

This is an early-stage skeleton. The protocol handlers (`handle_proposal`, `handle_vote`, `handle_timeout`) have the correct control flow stubbed out but do not yet accumulate state — QC/TC construction, view advancement, and commit logic are the next items to implement.

## License

MIT
