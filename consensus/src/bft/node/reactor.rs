use super::context::Context;
use crypto::hash::{Hash, EMPTY_HASH};
use config::Node;
use std::time::Duration;
use std::{borrow::Borrow, sync::Arc};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time;
use types::{Block, ProtocolMsg, Replica, Certificate, Transaction};

#[derive(PartialEq)]
enum Phase {
    Propose,
    DeliverPropose,
    End,
}

pub async fn reactor(
    config: &Node,
    is_client_apollo_enabled: bool,
    net_send: Sender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: Receiver<(Replica, ProtocolMsg)>,
    cli_send: Sender<Arc<Block>>,
    mut cli_recv: Receiver<Transaction>,
) {
    // Optimization to improve latency when the payloads are high
    let (send, mut recv) = channel(util::CHANNEL_SIZE);
    let mut cx = Context::new(config, net_send, send);
    cx.is_client_apollo_enabled = is_client_apollo_enabled;
    let block_size = config.block_size;
    let myid = config.id;
    let pl_size = config.payload;
    let delta = config.delta;
    // A little time to boot everything up
    let mut phase = Phase::End;
    let mut phase_end = time::sleep_until(time::Instant::now() + Duration::from_millis(delta * 11));
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                if let None = pmsg_opt {
                    log::error!(target:"node", "Protocol message channel closed");
                    std::process::exit(0);
                }
                let (_, pmsg) = pmsg_opt.unwrap();
                match pmsg {
                    ProtocolMsg::Certificate(p) => {
                        if myid == cx.last_leader && phase == Phase::Propose {
                            // TODO: Check that the certificate is valid
                            let hash = if p.votes.len() == 0 { EMPTY_HASH } else { p.votes[0].msg };
                            if let Some(block) = cx.storage.committed_blocks_by_hash.find(hash) {
                                if block.header.height > cx.highest_height {
                                    cx.highest_cert = p;
                                    cx.highest_height = block.header.height;
                                }
                            }
                        }
                    },
                    _ => {},
                };
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
            },
            _ = phase_end => {
                match phase {
                    Phase::End => {
                        cx.last_leader = cx.next_leader();
                        println!("{}: 11 delta has elapsed. Leader is {}.", myid, cx.last_leader);
                        if myid != cx.last_leader {
                            phase = Phase::DeliverPropose;
                            phase_end = time::sleep_until(time::Instant::now() + Duration::from_millis(delta * 7));
                            // Send the certification
                            cx.net_send.send((cx.last_leader, Arc::new(ProtocolMsg::Certificate(cx.last_seen_block.header.certificate.clone())))).await.unwrap();
                        } else {
                            highest_cert = Certificate::empty_cert()
                            highest_height = -1;
                            phase = Phase::Propose;
                            phase_end = time::sleep_until(time::Instant::now() + Duration::from_millis(delta * 2));
                        }
                    }
                }
            },
        }
    }
}
