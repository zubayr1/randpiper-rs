/// The core consensus module used for Apollo
///
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time;
// use crate::{Sender, Receiver};
// use crossfire::mpsc::bounded_tx_future_rx_blocking;
use super::blame::*;
use super::context::Context;
use super::proposal::*;
use config::Node;
use std::{borrow::Borrow, sync::Arc};
use std::time::{Duration, Instant};
use types::{Block, ProtocolMsg, Replica, Transaction};

pub async fn reactor(
    config: &Node,
    net_send: Sender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: Receiver<Arc<ProtocolMsg>>,
    cli_send: Sender<Arc<Block>>,
    mut cli_recv: Receiver<Arc<Transaction>>,
    is_client_apollo_enabled: bool,
) {
    let (send, mut recv) = channel(util::CHANNEL_SIZE);
    let mut cx = Context::new(config, net_send, send);
    cx.is_client_apollo_enabled = is_client_apollo_enabled;
    let block_size = config.block_size;
    let myid = config.id;
    let pl_size = config.payload;
    let delta = config.delta;
    let mut epoch_end = time::interval(Duration::from_millis(delta * 11));
    epoch_end.tick().await;
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
            },
        }
    }
}
