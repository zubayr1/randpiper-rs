use super::Certificate;
use crate::protocol::{Height, Replica};
use crate::Vote;
use crypto::hash::{Hash, EMPTY_HASH};
use serde::{Deserialize, Serialize};
use types_upstream::WireReady;

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    pub commits: Vec<crypto::EVSSCommit381>,
    pub acks: Vec<Vote>,
}

impl Content {
    pub const fn new() -> Self {
        Content {
            commits: Vec::new(),
            acks: Vec::new(),
        }

    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: Content = bincode::deserialize(&bytes).expect("failed to decode the content");
        return c;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockBody {
    pub data: Content,
}

impl BlockBody {
    pub const fn new() -> Self {
        BlockBody { data: Content::new() }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub prev: Hash,
    pub extra: Vec<u8>,
    pub author: Replica,
    pub height: Height,
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
            // .field("data", &self.data)
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
    pub certificate: Certificate,
}

impl Block {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let c: Block = bincode::deserialize(&bytes).expect("failed to decode the block");
        return c.init();
    }

    pub fn new() -> Self {
        Block {
            header: BlockHeader::new(),
            body: BlockBody::new(),
            hash: EMPTY_HASH,
            payload: Vec::new(),
            certificate: Certificate::empty_cert(),
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
        let empty_cert = Certificate::empty_cert();
        let old_cert = std::mem::replace(&mut self.certificate, empty_cert);
        self.hash = crypto::hash::ser_and_hash(&self);
        let _ = std::mem::replace(&mut self.payload, old_vec);
        let _ = std::mem::replace(&mut self.certificate, old_cert);
    }
}

pub const GENESIS_BLOCK: Block = Block {
    header: BlockHeader {
        prev: EMPTY_HASH,
        extra: Vec::new(),
        author: 0,
        height: 0,
    },
    body: BlockBody::new(),
    hash: EMPTY_HASH,
    payload: vec![],
    certificate: Certificate::empty_cert(),
    // cert: Certificate{
    // votes: vec![],
    // },
};

impl types_upstream::WireReady for Block {
    fn init(self) -> Self {
        self
    }

    fn from_bytes(data: &[u8]) -> Self {
        Block::from_bytes(data)
    }
}
