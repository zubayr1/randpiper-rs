extern crate consensus;
use crypto::rand::{rngs::StdRng, SeedableRng};
use crypto::*;
use std::collections::HashMap;
use types::{Block, Certificate, Content, Propose, Vote};
use util::io::to_bytes;

const SEED: u64 = 42;
static TEST_POINTS: [usize; 7] = [3, 10, 20, 30, 50, 75, 100];

fn generate_propose() -> HashMap<usize, Propose> {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut propose_map = HashMap::new();
    for test in &TEST_POINTS {
        let num_faults = (*test - 1) / 2;
        let params = EVSS381::setup(num_faults, rng).unwrap();
        let poly = EVSS381::commit(&params, F381::rand(rng), rng).unwrap();
        let mut certificate = Certificate::empty_cert();
        for _ in 0..*test {
            certificate.votes.push(Vote {
                msg: [0; 32].to_vec(),
                origin: 0,
                auth: [0; 32].to_vec(),
            })
        }
        let content = Content {
            acks: certificate.votes.clone(),
            commits: vec![poly.get_commit(); *test],
        };
        let mut block = Block::new();
        block.body.data = content;
        block.update_hash();
        let propose = Propose {
            new_block: block,
            certificate: certificate,
            epoch: 0,
        };
        propose_map.insert(*test, propose);
    }
    propose_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propose_length() {
        let p = generate_propose();
        for n in TEST_POINTS.iter() {
            println!("n = {}: propose_length = {}", n, to_bytes(p.get(&n).unwrap()).len());
        }
    }
}