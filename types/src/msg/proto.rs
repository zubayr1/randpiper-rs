use serde::{Deserialize, Serialize};

use super::Certificate;
use crate::{Propose, SignedData, Replica, Vote};
use types_upstream::WireReady;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    Certificate(Certificate),
    Propose(Propose, SignedData),
    Vote(Vote),
    VoteCert(Certificate, SignedData),
    DeliverPropose(Vec<u8>, Replica, SignedData),
    DeliverVoteCert(Vec<u8>, Replica, SignedData),
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: ProtocolMsg =
            flexbuffers::from_slice(&bytes).expect("failed to decode the protocol message");
        return c.init();
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
