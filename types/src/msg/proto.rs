use serde::{Deserialize, Serialize};

use super::Certificate;
use crate::{Propose, Height, Replica, SignedData, Vote};
use types_upstream::WireReady;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    Certificate(Certificate),
    Propose(Propose, SignedData),
    Vote(Vote),
    VoteCert(Certificate, SignedData),
    DeliverPropose(Vec<u8>, Replica, SignedData),
    DeliverVoteCert(Vec<u8>, Replica, SignedData),
    Reconstruct(crypto::EVSSShare381, Replica, Height),
    Shards(std::collections::VecDeque<crypto::EVSSShare381>),
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: ProtocolMsg =
            flexbuffers::from_slice(&bytes).expect("failed to decode the protocol message");
        return c.init();
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            ProtocolMsg::Certificate(_) => "Certificate",
            ProtocolMsg::Propose(_, _) => "Propose",
            ProtocolMsg::Vote(_) => "Vote",
            ProtocolMsg::VoteCert(_, _) => "VoteCert",
            ProtocolMsg::DeliverPropose(_, _, _) => "DeliverPropose",
            ProtocolMsg::DeliverVoteCert(_, _, _) => "DeliverVoteCert",
            ProtocolMsg::Reconstruct(_, _, _) => "Reconstruct",
            ProtocolMsg::Shards(_) => "Shards",
        }
    }
}

impl WireReady for ProtocolMsg {
    fn init(self) -> Self {
        self
    }

    fn from_bytes(data: &[u8]) -> Self {
        ProtocolMsg::from_bytes(data)
    }
}
