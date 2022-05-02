use crypto::hash::Hash;
use serde::{Serialize, Deserialize};

use crate::{Epoch, Height, Propose};

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    height: Height,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Certificate {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Accumulator {

}

#[derive(Debug, Serialize, Deserialize)]
pub enum Msg {
    Propose{
        block: Block, 
        e: Epoch, 
        cert: Certificate, 
        z_pe : Accumulator,
        sig: Vec<u8>,
    },
    Deliver{
        hash: Hash, 
        p_e: Hash,
        e: Epoch, 
        sig: Vec<u8>
    },
    Vote{
        hash: Hash,
        e: Epoch,
        sig: Vec<u8>,
    },
    VoteCert {
        cert: Certificate, 
        e: Epoch, 
        z_ve: Accumulator,
        sig: Vec<u8>,
    }
}

pub const GENESIS_BLOCK:Block = Block{
    height: 0,
};