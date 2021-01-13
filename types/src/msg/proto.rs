use serde::{Deserialize, Serialize};

use super::Certificate;
use crate::{Propose, Vote};
use types_upstream::WireReady;
use crypto::EVSSPublicParams381;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    Certificate(Certificate),
    Propose(Propose, EVSSPublicParams381),
    Vote(Vote),
    VoteCert(Certificate, EVSSPublicParams381),
    DeliverPropose(),
    DeliverVoteCert(),
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
