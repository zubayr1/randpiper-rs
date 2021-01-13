use super::context::Context;
use config::Node;
use crypto::hash::{ser_and_hash, EMPTY_HASH};
use crypto::rand::SeedableRng;
use crypto::{Biaccumulator381, F381};
use std::time::Duration;
use std::{convert::TryInto, sync::Arc};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time;
use types::{Block, Certificate, Height, Proof, Propose, ProtocolMsg, Replica, Transaction, Vote};
use reed_solomon_erasure::galois_8::ReedSolomon;

#[derive(PartialEq, Debug)]
enum Phase {
    Propose,
    DeliverPropose,
    Vote,
    Commit,
    End,
}

fn to_shards(data: &[u8], num_nodes: usize, num_faults: usize) -> Vec<Vec<u8>> {
    let num_data_shards = num_nodes - num_faults;
    let shard_size = (data.len() + num_data_shards - 1) / num_data_shards;
    let mut data_with_suffix = data.to_vec();
    let suffix_size = shard_size * num_data_shards - data.len();
    for _ in 0..suffix_size {
        data_with_suffix.push(suffix_size as u8)
    }
    let mut result = Vec::new();
    for shard in 0..shard_size {
        result.push(data_with_suffix[shard * shard_size..(shard + 1) * shard_size].to_vec());
    }
    let r = ReedSolomon::new(num_data_shards, num_faults).unwrap();
    r.encode(&mut result).unwrap();
    result
}

fn from_shards(data: &mut Vec<Option<Vec<u8>>>, num_nodes: usize, num_faults: usize) -> Vec<u8> {
    let num_data_shards = num_nodes - num_faults;
    let r = ReedSolomon::new(num_data_shards, num_faults).unwrap();
    r.reconstruct(data).unwrap();
    let mut result = Vec::new();
    for shard in 0..num_data_shards {
        result.append(&mut data[shard].clone().unwrap());
    }
    result.truncate(result.len() - *result.last().unwrap() as usize);
    result
}

pub async fn reactor(
    config: &Node,
    is_client_apollo_enabled: bool,
    net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: UnboundedReceiver<(Replica, ProtocolMsg)>,
    cli_send: UnboundedSender<Arc<Block>>,
    mut cli_recv: UnboundedReceiver<Transaction>,
) {
    // Optimization to improve latency when the payloads are high
    let (send, mut recv) = unbounded_channel();
    let mut cx = Context::new(config, net_send, send);
    cx.is_client_apollo_enabled = is_client_apollo_enabled;
    let block_size = config.block_size;
    let myid = config.id;
    let pl_size = config.payload;
    let delta = config.delta;
    let mut epoch: Height = 0;
    // A little time to boot everything up
    let begin = time::Instant::now() + Duration::from_millis(delta);
    let mut phase = Phase::End;
    let phase_end = time::sleep_until(begin);
    tokio::pin!(phase_end);
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                if let None = pmsg_opt {
                    log::error!(target:"node", "Protocol message channel closed");
                    std::process::exit(0);
                }
                let (_, pmsg) = pmsg_opt.unwrap();
                println!("{}: Received {:?}.", myid, pmsg);
                match pmsg {
                    ProtocolMsg::Certificate(p) => {
                        if myid == cx.last_leader && phase == Phase::Propose {
                            // TODO: Check that the certificate is valid
                            let hash = if p.votes.len() == 0 { EMPTY_HASH.to_vec() } else { p.votes[0].msg.clone() };
                            if let Some(block) = cx.storage.committed_blocks_by_hash.get(&TryInto::<[u8; 32]>::try_into(hash).unwrap()) {
                                if block.header.height > cx.highest_height {
                                    cx.highest_cert = p;
                                    cx.highest_height = block.header.height;
                                }
                            }
                        }
                    },
                    ProtocolMsg::Propose(p, z) => {
                        cx.received_propose = Some(p);
                        cx.reconstructed_propose = cx.received_propose.clone();
                    },
                    ProtocolMsg::Vote(p) => {
                        cx.received_vote.push(p);
                        if cx.received_vote.len() == (cx.num_faults + 1) as usize {
                            let certificate = Certificate {
                                votes: cx.received_vote,
                            };
                            cx.received_vote = Vec::new();
                            // TODO: Make a real setup
                            let setup = Biaccumulator381::setup(&[F381::from(0 as u32), F381::from(1 as u32), F381::from(2 as u32)], 3, &mut crypto::rand::rngs::StdRng::from_entropy()).unwrap();
                            cx.net_send.send((cx.num_nodes, Arc::new(ProtocolMsg::VoteCert(certificate.clone(), setup.get_public_params())))).unwrap();
                            cx.received_certificate = Some(certificate);
                            cx.reconstructed_certificate = cx.received_certificate.clone();
                            phase = Phase::Commit;
                            phase_end.as_mut().reset(time::Instant::now() + Duration::from_millis(delta * 2));
                        }
                    },
                    ProtocolMsg::VoteCert(c, z) => {
                        cx.received_certificate = Some(c);
                        cx.reconstructed_certificate = cx.received_certificate.clone();
                        phase = Phase::Commit;
                        phase_end.as_mut().reset(time::Instant::now() + Duration::from_millis(delta * 2));
                    },
                    _ => {}
                };
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
            },
            _ = &mut phase_end => {
                println!("{}: Phase {:?}", myid, phase);
                match phase {
                    Phase::Propose => {
                        let mut new_block = Block::new();
                        if cx.highest_cert.votes.len() == 0 {
                            new_block.header.prev = EMPTY_HASH;
                        } else {
                            new_block.header.prev = cx.highest_cert.votes[0].msg.clone().try_into().unwrap();
                        };
                        new_block.header.author = myid;
                        new_block.header.height = cx.highest_height + 1;
                        // TODO: Maybe add something to body?
                        new_block.update_hash();
                        // TODO: Make a real setup
                        let setup = Biaccumulator381::setup(&[F381::from(0 as u32), F381::from(1 as u32), F381::from(2 as u32)], 3, &mut crypto::rand::rngs::StdRng::from_entropy()).unwrap();
                        let proof = Proof {
                            block_hash: new_block.hash.to_vec(),
                            certificate_hash: ser_and_hash(&cx.highest_cert).to_vec(),
                            epoch: epoch,
                        };
                        let propose = Propose {
                            new_block: new_block,
                            certificate: cx.highest_cert,
                            sign: ser_and_hash(&proof).to_vec(),
                            proof: proof,
                        };
                        cx.highest_cert = Certificate::empty_cert();
                        cx.net_send.send((cx.num_nodes, Arc::new(ProtocolMsg::Propose(propose.clone(), setup.get_public_params())))).unwrap();
                        cx.received_propose = Some(propose);
                        cx.reconstructed_propose = cx.received_propose.clone();
                        phase = Phase::End;
                        phase_end.as_mut().reset(begin + Duration::from_millis(delta * 11 * epoch));
                    }
                    Phase::DeliverPropose => {
                        // TODO: Invoke Deliver and Reconstruct
                        phase = Phase::Vote;
                        phase_end.as_mut().reset(time::Instant::now() + Duration::from_millis(delta * 2));
                    }
                    Phase::Vote => {
                        // TODO: Check the propose
                        let propose = cx.reconstructed_propose.clone().unwrap();
                        let mut block = propose.new_block;
                        block.update_hash();
                        let vote = Vote {
                            msg: block.hash.to_vec(),
                            origin: myid,
                            auth: cx.my_secret_key.sign(&block.hash).unwrap(),
                        };
                        cx.net_send.send((cx.last_leader, Arc::new(ProtocolMsg::Vote(vote)))).unwrap();
                        phase = Phase::End;
                        phase_end.as_mut().reset(begin + Duration::from_millis(delta * 11 * epoch));
                    }
                    Phase::Commit => {
                        let new_block = Arc::new(cx.reconstructed_propose.unwrap().new_block);
                        cx.storage
                            .committed_blocks_by_hash
                            .insert(new_block.hash.clone(), Arc::clone(&new_block));
                        cx.storage
                            .committed_blocks_by_ht
                            .insert(new_block.header.height, Arc::clone(&new_block));
                        cx.received_propose = None;
                        cx.reconstructed_propose = None;
                        cx.received_certificate = None;
                        cx.reconstructed_certificate = None;
                        phase = Phase::End;
                        phase_end.as_mut().reset(begin + Duration::from_millis(delta * 11 * epoch));
                    }
                    Phase::End => {
                        cx.last_leader = cx.next_leader();
                        epoch += 1;
                        println!("{}: epoch {}. Leader is {}.", myid, epoch, cx.last_leader);
                        if myid != cx.last_leader {
                            // Send the certification
                            cx.net_send.send((cx.last_leader, Arc::new(ProtocolMsg::Certificate(cx.last_seen_block.certificate.clone())))).unwrap();
                            println!("{}: Certification sent.", myid);
                            phase = Phase::DeliverPropose;
                            phase_end.as_mut().reset(time::Instant::now() + Duration::from_millis(delta * 7));
                        } else {
                            cx.highest_cert = Certificate::empty_cert();
                            cx.highest_height = 0;
                            phase = Phase::Propose;
                            phase_end.as_mut().reset(time::Instant::now() + Duration::from_millis(delta * 2));
                        }
                    }
                }
            },
        }
    }
}
