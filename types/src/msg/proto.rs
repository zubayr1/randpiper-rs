use serde::{Deserialize, Serialize};

use super::Certificate;
use crate::{Propose, Replica, Vote, WireReady};
use crypto::EVSSPublicParams381;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    Certificate(Certificate),
    Propose(Propose),
    Vote(Vote),
    VoteCert(Certificate, EVSSPublicParams381),
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let c: ProtocolMsg =
            flexbuffers::from_slice(&bytes).expect("failed to decode the protocol message");
        return c;
    }
}

impl WireReady for ProtocolMsg {}
