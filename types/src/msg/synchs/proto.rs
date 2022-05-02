use serde::{Deserialize, Serialize};

use crate::{msg::block::Block, synchs::Propose, Certificate, Replica, View, Vote};
use types_upstream::WireReady;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    /// Identification message to tell the other node that I am node ID
    Identify(Replica),
    /// New Proposal
    NewProposal(Propose),
    /// Can be a blame or a vote
    Vote(Vote),
    /// VoteMsg because a vote needs to have a block
    VoteMsg(Vote, Propose),
    /// Certificate saying that all the nodes are waiting to quit the view
    QuitView(View, Certificate),
    /// Status: Contains the block and its certificate
    Status(Block, Certificate),
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: ProtocolMsg =
            bincode::deserialize(&bytes).expect("failed to decode the protocol message");
        return c.init();
    }

    pub fn init(self) -> Self {
        match self {
            ProtocolMsg::NewProposal(mut p) => {
                p.init();
                return ProtocolMsg::NewProposal(p);
            }
            ProtocolMsg::VoteMsg(_v, mut p) => {
                p.init();
                return ProtocolMsg::VoteMsg(_v, p);
            }
            _ => (),
        }
        self
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
