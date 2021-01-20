extern crate consensus;
use consensus::bft::node::accumulator;
use crypto::rand::{rngs::StdRng, SeedableRng};
use crypto::*;
use serde::Serialize;
use std::collections::HashMap;
use types::*;
use util::io::to_bytes;

const SEED: u64 = 42;
static TEST_POINTS: [usize; 98] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75,
    76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99,
    100,
];

fn tree_get_dummy_acc<T: Serialize>(
    cx_num_nodes: Replica,
    cx_num_faults: Replica,
    data: &T,
) -> (Vec<Vec<u8>>, DataWithAcc) {
    let shards = accumulator::to_shards(
        &to_bytes(data),
        cx_num_nodes as usize,
        cx_num_faults as usize,
    );
    let size = accumulator::get_size(cx_num_nodes) as usize;
    let mut tree = vec![Vec::new(); (1 << size) + 1];
    for i in 0..cx_num_nodes as usize {
        tree[1 << size - 1 | i] = hash::ser_and_hash(&shards[i]).to_vec();
    }
    for i in 0..(1 << size - 1) - 1 {
        let index = (1 << size - 1) - 1 - i;
        tree[index] =
            hash::ser_and_hash(&(tree[index << 1].clone(), tree[index << 1 | 1].clone())).to_vec();
    }
    (
        shards,
        DataWithAcc {
            sign: Vec::new(),
            tree: tree,
            size: size as Replica,
        },
    )
}

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

    fn empty_vote() -> Vote {
        Vote {
            msg: [0 as u8; 32].to_vec(),
            origin: 0,
            auth: [0 as u8, 32].to_vec(),
        }
    }

    #[test]
    fn certificate_length() {
        for n in TEST_POINTS.iter() {
            let data = Certificate {
                votes: vec![empty_vote(); *n],
            };
            println!(
                "Certificate,{},{}",
                n,
                to_bytes(&ProtocolMsg::Certificate(data)).len()
            );
        }
    }

    #[test]
    fn propose_length() {
        let p = generate_propose();
        for n in TEST_POINTS.iter() {
            let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, p.get(&n).unwrap()).1;
            println!(
                "Propose,{},{}",
                n,
                to_bytes(&ProtocolMsg::Propose(p.get(&n).unwrap().clone(), acc)).len()
            );
        }
    }

    #[test]
    fn vote_length() {
        for n in TEST_POINTS.iter() {
            println!(
                "Vote,{},{}",
                n,
                to_bytes(&ProtocolMsg::Vote(empty_vote())).len()
            );
        }
    }

    #[test]
    fn vote_cert_length() {
        for n in TEST_POINTS.iter() {
            let data = Certificate {
                votes: vec![empty_vote(); *n],
            };
            let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, &data).1;
            println!(
                "VoteCert,{},{}",
                n,
                to_bytes(&ProtocolMsg::VoteCert(data, acc)).len()
            );
        }
    }

    #[test]
    fn deliver_propose_length() {
        let p = generate_propose();
        for n in TEST_POINTS.iter() {
            let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, p.get(&n).unwrap());
            println!(
                "DeliverPropose,{},{}",
                n,
                to_bytes(&ProtocolMsg::DeliverPropose(
                    acc.0[0].clone(),
                    0,
                    accumulator::get_sign(&acc.1, 0)
                ))
                .len()
            );
        }
    }

    #[test]
    fn deliver_vote_cert_length() {
        for n in TEST_POINTS.iter() {
            let data = Certificate {
                votes: vec![empty_vote(); *n],
            };
            let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, &data);
            println!(
                "DeliverVoteCert,{},{}",
                n,
                to_bytes(&ProtocolMsg::DeliverPropose(
                    acc.0[0].clone(),
                    0,
                    accumulator::get_sign(&acc.1, 0)
                ))
                .len()
            );
        }
    }

    #[test]
    fn reconstruct_length() {
        for n in TEST_POINTS.iter() {
            let _num_nodes = *n;
            let num_faults = (n - 1) / 2;
            let rng = &mut StdRng::from_entropy();
            let rand_beacon_parameter = crypto::EVSS381::setup(num_faults, rng).unwrap();
            let poly =
                crypto::EVSS381::commit(&rand_beacon_parameter, crypto::F381::rand(rng), rng)
                    .unwrap();
            let share = crypto::EVSS381::get_share(
                crypto::F381::from(0 as u16),
                &rand_beacon_parameter,
                &poly,
                rng,
            )
            .unwrap();
            println!(
                "Reconstruct,{},{}",
                n,
                to_bytes(&ProtocolMsg::Reconstruct(share, 0)).len()
            );
        }
    }

    #[test]
    fn commit_length() {
        for n in TEST_POINTS.iter() {
            let num_nodes = *n;
            let num_faults = (n - 1) / 2;
            let rng = &mut StdRng::from_entropy();
            let rand_beacon_parameter = crypto::EVSS381::setup(num_faults, rng).unwrap();
            let mut shares = vec![std::collections::VecDeque::with_capacity(num_nodes); num_nodes];
            let mut commits = Vec::with_capacity(num_nodes);
            for _ in 0..num_nodes {
                let poly =
                    crypto::EVSS381::commit(&rand_beacon_parameter, crypto::F381::rand(rng), rng)
                        .unwrap();
                commits.push(poly.get_commit());
                for j in 0..1 {
                    shares[j].push_back(
                        crypto::EVSS381::get_share(
                            crypto::F381::from((j + 1) as u16),
                            &rand_beacon_parameter,
                            &poly,
                            rng,
                        )
                        .unwrap(),
                    );
                }
            }
            let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, &to_bytes(&commits));
            println!(
                "Commit,{},{}",
                n,
                to_bytes(&ProtocolMsg::Commit(shares[0].clone(), commits, acc.1)).len()
            );
        }
    }

    #[test]
    fn deliver_commit_length() {
        for n in TEST_POINTS.iter() {
            let num_nodes = *n;
            let num_faults = (n - 1) / 2;
            let rng = &mut StdRng::from_entropy();
            let rand_beacon_parameter = crypto::EVSS381::setup(num_faults, rng).unwrap();
            let mut shares = vec![std::collections::VecDeque::with_capacity(num_nodes); num_nodes];
            let mut commits = Vec::with_capacity(num_nodes);
            for _ in 0..num_nodes {
                let poly =
                    crypto::EVSS381::commit(&rand_beacon_parameter, crypto::F381::rand(rng), rng)
                        .unwrap();
                commits.push(poly.get_commit());
                for j in 0..1 {
                    shares[j].push_back(
                        crypto::EVSS381::get_share(
                            crypto::F381::from((j + 1) as u16),
                            &rand_beacon_parameter,
                            &poly,
                            rng,
                        )
                        .unwrap(),
                    );
                }
            }
            let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, &to_bytes(&commits));
            println!(
                "DeliverCommit,{},{}",
                n,
                to_bytes(&ProtocolMsg::DeliverCommit(
                    acc.0[0].clone(),
                    0,
                    accumulator::get_sign(&acc.1, 0)
                ))
                .len()
            );
        }
    }

    #[test]
    fn ack_length() {
        for n in TEST_POINTS.iter() {
            println!(
                "Ack,{},{}",
                n,
                to_bytes(&ProtocolMsg::Ack(empty_vote())).len()
            );
        }
    }
}
