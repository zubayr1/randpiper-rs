use super::Block;
use crate::{Certificate, Height};
use crypto::EVSSPublicParams381;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Proof {
    pub block_hash: Vec<u8>,
    pub certificate_hash: Vec<u8>,
    pub epoch: Height,
    pub accumulator: EVSSPublicParams381,
}

#[derive(Serialize, Deserialize)]
pub struct Propose {
    pub new_block: Block,
    pub certificate: Certificate,
    pub proof: Proof,
    pub sign: Vec<u8>,
}

impl std::fmt::Debug for Propose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Propose").finish()
    }
}

impl Clone for Proof {
    fn clone(&self) -> Self {
        Proof {
            block_hash: self.block_hash.clone(),
            certificate_hash: self.certificate_hash.clone(),
            epoch: self.epoch.clone(),
            accumulator: EVSSPublicParams381 {
                verifier_key: self.accumulator.verifier_key.clone(),
                commit: self.accumulator.commit.clone(),
            },
        }
    }
}

impl Clone for Propose {
    fn clone(&self) -> Self {
        Propose {
            new_block: self.new_block.clone(),
            certificate: self.certificate.clone(),
            proof: self.proof.clone(),
            sign: self.sign.clone(),
        }
    }
}
