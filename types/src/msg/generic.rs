use serde::{Deserialize, Serialize};

use crate::{protocol::*};
use types_upstream::WireReady;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    pub msg: Vec<u8>,
    pub origin: Replica,
    pub auth: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Certificate {
    pub votes: Vec<Vote>,
}

impl Certificate {

    pub const fn empty_cert() -> Self {
        Certificate { votes: Vec::new() }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: Certificate =
            flexbuffers::from_slice(&bytes).expect("failed to decode the propose");
        c
    }

}

impl std::default::Default for Certificate {
    fn default() -> Self {
        Certificate::empty_cert()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub data: Vec<u8>,
    pub request: Vec<u8>,
}

impl Transaction {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: Transaction = flexbuffers::from_slice(&bytes).expect("failed to decode the block");
        return c.init();
    }
}

impl WireReady for Transaction {
    fn init(self) -> Self {
        self
    }

    fn from_bytes(data: &[u8]) -> Self {
        Transaction::from_bytes(data)
    }

}
