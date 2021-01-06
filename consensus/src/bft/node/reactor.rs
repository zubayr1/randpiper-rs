/// The core consensus module used for Apollo
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::time;
use types::{Block, Replica, ProtocolMsg, Transaction};
use config::Node;
use super::context::Context;
use std::time::{Duration, Instant};
use std::{sync::Arc, borrow::Borrow};

pub async fn reactor(
    config:&Node,
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
    let cli_send_p = cli_send.clone();
    let delta = config.delta;
    let mut epoch_end = time::interval(Duration::from_millis(delta * 11));
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
            },
            _ = epoch_end.tick() => {
                cx.last_leader = cx.next_leader();
                println!("{}: 11 delta has elapsed. Leader is {}.", myid, cx.last_leader);
                // Send the certification
            },
        }
    }
}