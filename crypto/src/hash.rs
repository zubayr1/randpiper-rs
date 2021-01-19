use serde::Serialize;
use sha2::{Digest, Sha256};

pub const HASH_SIZE: usize = 32;
pub type Hash = [u8; HASH_SIZE];

pub const EMPTY_HASH: Hash = [0 as u8; 32];

pub fn do_hash(bytes: &[u8]) -> Hash {
    let hash = Sha256::digest(bytes);
    return hash.into();
}

pub fn ser_and_hash(obj: &impl Serialize) -> Hash {
    return do_hash(&bincode::serialize(&obj).unwrap());
}
