use std::collections::HashMap;
use std::marker::PhantomData;

use crate::config::Config;
use crate::crypto::{CryptoProvider, Signer};
use crate::error::ConsensusError;
use crate::message::{Justification, Message, Proposal, Vote, Finalize};
use crate::types::{Digest, NodeId, Height};

/// Entry-point for a Simplex consensus replica.
///
/// # Type parameters
///
/// * `P` — the opaque payload / command type.
/// * `N` — the network transport (see [`crate::network::Network`]).
/// * `S` — the application state-machine (see [`crate::state::StateMachine`]).
/// * `C` — cryptographic primitives (see [`crate::crypto::CryptoProvider`]).
/// * `K` — the signing key for this replica (see [`crate::crypto::Signer`]).
pub struct Simplex<P, N, S, C, K> {
    config: Config,
    network: N,
    state_machine: S,
    crypto: C,
    signer: K,

    // todo: review below protocol state.

    // todo: add the WAL (write ahead log) for the engine, missing WAL might cause safety issue for
    // the engine. Here is a scenario:
    // There are 4 nodes: A, B, C, D on the same view V_n.
    // At the next consensus instance, the protocol have made decision to commit P1 to V_n+1;
    // However a disaster causes:
    // A commits P1 to V_n+1, but B, C, D crash before they commit P1 to V_n+1.
    // After the disaster, A is the only node that has committed P1 to V_n+1, and B, C, D are still on V_n.
    // If A crashes without a recovery, then B, C, D recovery to make new decision P2 to V_n+1,
    // which will cause a safety issue, because A has already committed P1 to V_n+1.

    // The WAL should base on a MMap file which bypass the context switching to reduce the latency,
    // and the WAL should be flushed to disk before the commit decision is made, so that if a
    // disaster happens, the WAL can be used to recover the state of the engine.

    // --- Protocol state ---
    /// The current view.
    view: Height,
    /// The highest sequence that has been committed.
    committed_seq: Height,
    /// The digest of the last committed state.
    locked_digest: Digest,
    /// The highest `(view, seq)` this replica has voted for.
    highest_vote: Option<(Height, Height)>,
    /// Buffer of received timeout messages per view, used to build [`TimeoutCert`].
    timeout_buffer: HashMap<Height, Vec<Vote>>,

    /// Marker — the engine does not store payloads directly, but the type flows
    /// through all message handlers.
    _phantom: PhantomData<P>,
}

impl<P, N, S, C, K> Simplex<P, N, S, C, K>
where
    P: Clone + Send + Sync + 'static + serde::Serialize,
    N: crate::network::Network<P>,
    S: crate::state::StateMachine<P>,
    C: CryptoProvider,
    K: Signer,
{
    /// Create a fresh simplex instance.
    pub fn new(config: Config, network: N, state_machine: S, crypto: C, signer: K) -> Self {
        let locked_digest = state_machine.digest();
        Self {
            config,
            network,
            state_machine,
            crypto,
            signer,
            view: 0,
            committed_seq: 0,
            locked_digest,
            highest_vote: None,
            timeout_buffer: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    // ------------------------------------------------------------------
    //  Public entry-points
    // ------------------------------------------------------------------

    /// Called when a protocol message is received from the network.
    pub fn on_message(
        &mut self,
        from: NodeId,
        msg: Message<P>,
    ) -> Result<(), ConsensusError> {
        match msg {
            Message::Proposal(p) => self.handle_proposal(from, p),
            Message::Vote(v) => self.handle_vote(from, v),
            Message::Finalize(f) => self.handle_finalize(from, f),
        }
    }

    /// Start a new view — typically called after a timeout or on bootstrap.
    ///
    /// If `self` is the leader of the new view it will construct and broadcast a proposal.
    pub fn start_view(&mut self, view: Height) -> Result<(), ConsensusError> {
        self.view = view;
        if self.config.replica_set.is_leader_of(self.signer.node_id(), view) {
            self.propose(view)?;
        }
        // todo: for every P, it should start the timer of 3 Delta for liveness.
        // If there is no decision being made within 3 Delta, they should send a vote for dummy block
        // to the network, and if there are over quorum vote for dummy block, view change is triggerd.

        Ok(())
    }

    /// Current view of this replica.
    pub fn view(&self) -> Height {
        self.view
    }

    /// The last committed sequence number.
    pub fn committed_sequence(&self) -> Height {
        self.committed_seq
    }

    // ------------------------------------------------------------------
    //  Internal handlers
    // ------------------------------------------------------------------

    fn handle_proposal(
        &mut self,
        _from: NodeId,
        proposal: Proposal<P>,
    ) -> Result<(), ConsensusError> {
        // todo: if the proposal isn't from the leader of the current view, emit accountability
        // event.

        // todo: if we already voted for current view, skip voting for any other proposal.

        // todo: the proposal should be corrected formed.

        // Pipeline optimized by merging the last view's commit and current view's propose phase.
        // todo: verify the certificate of the last view's proposal, if its valid, commit the
        // last proposal, and then verify the new proposal of current view, if it is valid, vote for it.
        // 1. Verify the justification.
        self.verify_justification(&proposal.justification, proposal.height)?;

        // 2. Validate payload against the state machine.
        for cmd in &proposal.payload {
            self.state_machine.validate(cmd)?;
        }

        // 3. If safe, vote for the proposal.
        let vote = self.build_vote(&proposal)?;
        self.network.broadcast(Message::Vote(vote))?;

        Ok(())
    }

    fn handle_finalize(&mut self, _from: NodeId, finalize: Finalize) -> Result<(), ConsensusError> {
        // todo: implement finalize handling
        Ok(())
    }

    fn handle_vote(&mut self, _from: NodeId, vote: Vote) -> Result<(), ConsensusError> {
        // todo: handle timeout by processing the vote for dummy block.

        // 1. Verify the vote signature.
        self.crypto.verify(
            vote.voter,
            &self.vote_digest_bytes(&vote.view, &vote.digest),
            &vote.signature,
        )?;

        // 2. Accumulate votes; if quorum is reached, commit.
        //    (Simplified — full impl builds a QC from collected votes.)
        let _ = vote; // stub: real impl stores votes and checks for quorum
        Ok(())
    }

    fn handle_timeout(&mut self, _from: NodeId, timeout: Vote) -> Result<(), ConsensusError> {
        // 1. Verify the timeout signature.
        self.crypto.verify(
            timeout.sender,
            &self.timeout_digest_bytes(&timeout),
            &timeout.signature,
        )?;

        // 2. Accumulate timeouts; if `2f+1` are collected, build a TC and advance view.
        self.timeout_buffer
            .entry(timeout.next_view)
            .or_default()
            .push(timeout);

        // Stub: check for quorum and advance view if so.
        Ok(())
    }

    // ------------------------------------------------------------------
    //  Helpers
    // ------------------------------------------------------------------

    /// The leader constructs and broadcasts a new proposal for the given view.
    fn propose(&mut self, view: Height) -> Result<(), ConsensusError> {
        let proposal = Proposal {
            height: view,
            parent: self.locked_digest,
            payload: Vec::new(), // Host feeds pending commands here.
            justification: self.build_justification(view), // last decision's certificate
        };
        self.network.broadcast(Message::Proposal(proposal))?;
        Ok(())
    }

    // todo: build vote for timeout by dummy block hash.
    fn build_vote(&self, proposal: &Proposal<P>) -> Result<Vote, ConsensusError> {
        let digest = self.crypto.hash(&bincode_like_serialize(proposal));
        let sig = self.signer.sign(&self.vote_digest_bytes(&proposal.height, &digest))?;
        Ok(Vote {
            height: proposal.height,
            digest,
            voter: self.signer.node_id(),
            signature: sig,
        })
    }

    fn build_justification(&self, _view: Height) -> Justification {
        // Real impl: return the highest QC or TC known.
        Justification::Genesis
    }

    fn verify_justification(
        &self,
        _jc: &Justification,
        _view: Height,
    ) -> Result<(), ConsensusError> {
        // Real impl: verify the QC/TC signatures.
        Ok(())
    }

    fn vote_digest_bytes(&self, view: &Height, digest: &Digest) -> Vec<u8> {
        let mut out = view.to_le_bytes().to_vec();
        out.extend_from_slice(digest);
        out
    }

    fn timeout_digest_bytes(&self, t: &Vote) -> Vec<u8> {
        let mut out = t.current_view.to_le_bytes().to_vec();
        out.extend_from_slice(&t.next_view.to_le_bytes());
        if let Some((v, s)) = &t.high_vote {
            out.extend_from_slice(&v.to_le_bytes());
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }
}

/// Placeholder for a real serialization scheme (e.g. bincode or postcard).
fn bincode_like_serialize<T: serde::Serialize>(v: &T) -> Vec<u8> {
    bincode::serialize(v).unwrap_or_default()
}

// todo: implement a timer that can emit timeout event and cancel a timer as well, in this context,
// we need to start the timer with 3 delta on the start of a view, once we made the decision to
// commit a proposal, we need to cancel the timer, and if the timer expires, we need to send a
// timeout message to the network to form a quorum timeout certificate for liveness.

// todo: view change for slow client.
//  if there are over quorum votes of specific higher view than the local validator, then the local
//  might be a slow client, we need to change the view to catch up to the faster clients.
// todo: view change from the underlying blockchain execution layer.
//  if the execution layer already applied higher view from p2p synchronization layer, we need to
//  change the view to catch up to the execution layer as well. Thus, we need a subscriber to subscribe
//  the view change event from the state machine (the underlying blockchain), and if the view
//  is higher than the local view, we need to change the view to catch up.