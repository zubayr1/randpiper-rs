use serde::{Deserialize, Serialize};

use super::Certificate;
use crate::{Propose, Height, Replica, DataWithAcc, SignedData, Vote};
use types_upstream::WireReady;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    Certificate(Certificate),
    Propose(Propose, DataWithAcc),
    Vote(Vote),
    VoteCert(Certificate, DataWithAcc),
    DeliverPropose(Vec<u8>, Replica, SignedData),
    DeliverVoteCert(Vec<u8>, Replica, SignedData),
    Reconstruct(crypto::EVSSShare381, Height),
    Commit(std::collections::VecDeque<crypto::EVSSShare381>, Vec<crypto::EVSSCommit381>, DataWithAcc),
    DeliverCommit(Vec<u8>, Replica, SignedData),
    Ack(Vote),
}

pub fn commit_from_bytes(bytes: &[u8]) -> Vec<crypto::EVSSCommit381> {
    let c: Vec<crypto::EVSSCommit381> = bincode::deserialize(&bytes).expect("failed to decode the commit");
    c
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: ProtocolMsg =
            bincode::deserialize(&bytes).expect("failed to decode the protocol message");
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
            ProtocolMsg::Reconstruct(_, _) => "Reconstruct",
            ProtocolMsg::Commit(_, _, _) => "Commit",
            ProtocolMsg::DeliverCommit(_, _, _) => "DeliverCommit",
            ProtocolMsg::Ack(_) => "Ack",
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
