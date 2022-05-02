use super::Block;
use crate::{Certificate, Height};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Propose {
    pub new_block: Block,
    pub certificate: Certificate,
    pub epoch: Height,
}

impl Propose {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: Propose = bincode::deserialize(&bytes).expect("failed to decode the propose");
        c
    }
}
