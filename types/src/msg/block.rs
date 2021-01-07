use super::{Certificate, Transaction};
use crate::{
    protocol::{Height, Replica},
    WireReady,
};
use crypto::hash::{Hash, EMPTY_HASH};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockBody {
    pub data: Vec<u8>,
}

impl BlockBody {
    pub fn new() -> Self {
        BlockBody { data: Vec::new() }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub prev: Hash,
    pub extra: Vec<u8>,
    pub author: Replica,
    pub height: Height,
    pub certificate: Certificate,
}

impl std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block Header")
            .field("author", &self.author)
            .field("height", &self.height)
            .field("prev", &self.prev)
            .finish()
    }
}

impl std::fmt::Debug for BlockBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block Body")
            .field("data", &self.data)
            .finish()
    }
}

impl BlockHeader {
    pub fn new() -> Self {
        BlockHeader {
            prev: EMPTY_HASH,
            extra: Vec::new(),
            author: 0,
            height: 0,
            certificate: Certificate::empty_cert(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,

    #[serde(skip_serializing, skip_deserializing)]
    pub hash: Hash,
    // #[serde(skip_serializing, skip_deserializing)]
    pub payload: Vec<u8>,
}

impl Block {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let c: Block = flexbuffers::from_slice(&bytes).expect("failed to decode the block");
        return c;
    }

    pub fn new() -> Self {
        Block {
            header: BlockHeader::new(),
            body: BlockBody::new(),
            hash: EMPTY_HASH,
            payload: Vec::new(),
        }
    }

    pub fn add_payload(&mut self, payload: usize) {
        for i in 0..payload {
            self.payload.push(i as u8);
        }
    }

    pub fn update_hash(&mut self) {
        let empty_vec = vec![0; 0];
        let old_vec = std::mem::replace(&mut self.payload, empty_vec);
        self.hash = crypto::hash::ser_and_hash(&self);
        let _ = std::mem::replace(&mut self.payload, old_vec);
    }
}

pub const GENESIS_BLOCK: Block = Block {
    header: BlockHeader {
        prev: EMPTY_HASH,
        extra: Vec::new(),
        author: 0,
        height: 0,
        certificates: Certificate::empty_cert(),
    },
    body: BlockBody { data: Vec::new() },
    hash: EMPTY_HASH,
    payload: vec![],
    // cert: Certificate{
    // votes: vec![],
    // },
};

impl WireReady for Block {}
