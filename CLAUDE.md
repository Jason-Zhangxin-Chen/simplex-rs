# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project overview

`simplex-rs` is a Rust implementation of the **Simplex BFT consensus** protocol — a leader-based, partially-synchronous Byzantine Fault Tolerant consensus algorithm.

**Papers:**

- [Simplex Consensus (original)](https://eprint.iacr.org/2023/463.pdf)
- [Improved Simplex](https://eprint.iacr.org/2023/1916.pdf)

The crate abstracts the underlying blockchain and network layers behind traits, so the core protocol can be reused across different systems.

## Build & test

```bash
cargo build              # debug build
cargo build --release    # release build
cargo test               # run all tests
cargo clippy             # lint (if installed)
```

Rust edition is **2024** (requires Rust ≥ 1.85; current toolchain is 1.96).

## Architecture

The crate is a **library** (no binary target). The consensus engine `Simplex<P, N, S, C, K>` is generic over five type parameters, but conceptually they fall into four pluggable concerns:

| Type param | Trait | File | Role |
|---|---|---|---|
| `P` | (opaque) | — | Payload/command/transaction blob — the engine never interprets it |
| `N` | `Network<P>` | `src/network.rs` | Broadcast + point-to-point send |
| `S` | `StateMachine<P>` | `src/state.rs` | Validate, apply, and return digests of the replicated ledger |
| `C` | `CryptoProvider` | `src/crypto.rs` | Hash + signature verification (never touches private keys) |
| `K` | `Signer` | `src/crypto.rs` | Produce signatures for this replica's outgoing messages |

All these traits require `Send + Sync` — the engine is designed to be driven from async contexts (typically tokio).

## Module map

```
src/
├── lib.rs          # Crate root: declares all modules, re-exports key types
├── types.rs        # NodeId, View, Sequence, Digest, CommitteeSet
├── config.rs       # Config struct — tunable parameters (timeouts, batch limits)
├── error.rs        # ConsensusError enum (thiserror)
├── message.rs      # Wire-format types: Message<P>, Proposal<P>, Vote, Timeout,
│                   #   Justification (QC / TC / Genesis), QuorumCert, TimeoutCert
├── crypto.rs       # Traits: CryptoProvider (verify, hash), Signer (sign)
├── network.rs      # Traits: Network<P> (broadcast, send_to), MessageHandler<P>
├── state.rs        # Trait: StateMachine<P> (validate, apply, digest)
└── protocol.rs     # The main engine: Simplex<P,N,S,C,K>
```

## Key naming conventions

- **`CommitteeSet`** — set of validators, NOT "ReplicaSet". Contains `nodes: Vec<NodeId>` and `f: usize` (fault tolerance bound). Provides `n()`, `quorum_size()` (2f+1), `leader_of(view)`, `is_leader_of(id, view)`.
- **`View`** — what some papers call a "round" or "epoch". Monotonically increasing.
- **`Sequence`** — block height / position in the total order.

## Protocol flow (high-level)

The Simplex protocol proceeds in views. Each view has one designated leader (round-robin over `CommitteeSet::leader_of`):

1. **Bootstrap / view change:** `start_view(v)` is called. If this node is the leader, it builds and broadcasts a `Proposal`.
2. **Proposal handling:** `on_message(Proposal)` verifies the attached justification (QC, TC, or Genesis), validates the payload against the `StateMachine`, then votes.
3. **Vote handling:** `on_message(Vote)` verifies the signature and accumulates votes. When `2f+1` votes for the same proposal digest are gathered, a QC is formed and the proposal can be committed.
4. **Timeout handling:** `on_message(Timeout)` verifies the signature and accumulates timeouts. When `2f+1` timeouts for the same `next_view` are gathered, a TC is formed and the replica advances its view.

There is a planned **pipeline optimization** (noted in `protocol.rs`): the last view's commit phase and the current view's propose phase can be merged — verify the previous proposal's QC, commit it, then immediately verify and vote for the current proposal, all within the same `handle_proposal` call.

## Implementation status

The skeleton is in place but the core protocol logic is **stubbed**. Specific `todo`s marked in `protocol.rs`:

- **`handle_proposal`:** does not yet check that the sender is the leader of the current view. Does not emit accountability events for misbehaving leaders. Does not skip voting if already voted in this view. The pipeline optimization (commit last + vote current) is not wired — it naively votes for every valid proposal.
- **`handle_vote`:** signature verification is called, but votes are not stored and quorum is not checked. The `let _ = vote;` stub discards the vote.
- **`handle_timeout`:** timeouts are buffered in `timeout_buffer` but quorum is never checked, so TCs are never built and views never advance due to timeouts.
- **`start_view`:** non-leader nodes should start a `3Δ` timer for liveness (noted but not implemented).
- **`build_justification`:** always returns `Justification::Genesis`. Should return the highest known QC or TC.
- **`verify_justification`:** always returns `Ok(())`. Should verify the QC/TC signatures and quorum counts.
- **`propose`:** always sends an empty payload. The host needs a way to feed pending commands into the engine (the `MessageHandler` / network loop could push them, or a separate method could be added).

**Ergonomics to-do:** `SignatureBytes` is currently `Vec<u8>` — could be generic over a signature type for type-safety.

## Code style

- Edition 2024 Rust — use current idioms.
- All public types derive `Debug, Clone`.
- Wire types derive `Serialize, Deserialize` (serde).
- Errors use `thiserror`.
- The `P` type parameter uses `PhantomData<P>` in the struct since the engine doesn't store payloads directly; the trait bounds on the impl block (`P: Clone + Send + Sync + 'static + Serialize`) keep it constrained.
- Comments on struct fields and public methods are `///` doc comments.
- Implementation notes and TODOs use `// todo:` so they're easily grepable.
