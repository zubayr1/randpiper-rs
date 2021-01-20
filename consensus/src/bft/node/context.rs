use super::accumulator::ShareGatherer;

use std::collections::HashMap;

// use crossfire::mpsc::{SharedSenderFRecvB, TxFuture};
use crypto_lib::{ed25519, secp256k1, Keypair, PublicKey};
use tokio::sync::mpsc::UnboundedSender;
// use crate::Sender;
use config::Node;
use std::sync::Arc;
use types::{
    Block, Certificate, Height, Propose, ProtocolMsg, Replica, DataWithAcc, Storage, Vote,
    GENESIS_BLOCK,
};

// type Sender<T> = TxFuture<T, SharedFutureBoth>;

pub struct Context {
    pub num_nodes: u16,
    pub num_faults: u16,
    pub myid: Replica,
    pub pub_key_map: HashMap<Replica, PublicKey>,
    pub my_secret_key: Keypair,
    pub net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    pub cli_send: UnboundedSender<Block>,
    pub is_client_apollo_enabled: bool,

    pub storage: Storage,
    pub height: Height,
    pub epoch: Height,
    pub last_leader: Replica,
    pub last_seen_block: Arc<Block>,
    pub last_committed_block_ht: Height,
    pub payload: usize,

    pub highest_cert: Certificate,
    pub highest_height: Height,

    pub received_propose: Option<Propose>,
    pub received_propose_sign: Option<DataWithAcc>,

    pub received_vote: Vec<Vote>,
    pub received_ack: Vec<Vote>,

    pub received_certificate: Option<Certificate>,
    pub received_certificate_sign: Option<DataWithAcc>,

    pub received_commit: Option<Vec<crypto::EVSSCommit381>>,
    pub received_commit_sign: Option<DataWithAcc>,

    pub accumulator_pub_params_map: HashMap<Replica, crypto::EVSSPublicParams381>,
    pub accumulator_params: crypto::EVSSParams381,

    pub propose_share_sent: bool,
    pub vote_cert_share_sent: bool,
    pub commit_share_sent: bool,

    pub propose_gatherer: ShareGatherer,
    pub vote_cert_gatherer: ShareGatherer,
    pub commit_gatherer: ShareGatherer,
    
    pub rand_beacon_parameter: crypto::EVSSParams381,
    pub rand_beacon_queue: HashMap<Replica, std::collections::VecDeque<crypto::EVSSShare381>>,

    pub reconstruct_queue: std::collections::VecDeque<(crypto::EVSSShare381, Height)>,

    pub shards: Vec<std::collections::VecDeque<crypto::EVSSShare381>>,
    pub commits: Vec<crypto::EVSSCommit381>,

    pub rand_beacon_shares: Vec<(Vec<std::collections::VecDeque<crypto::EVSSShare381>>, Vec<crypto::EVSSCommit381>)>,
}

const EXTRA_SPACE: usize = 100;

impl Context {
    pub fn new(
        config: &Node,
        net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
        cli_send: UnboundedSender<Block>,
    ) -> Self {
        let genesis_block = Arc::new(GENESIS_BLOCK);
        let mut c = Context {
            num_nodes: config.num_nodes as u16,
            num_faults: config.num_faults as u16,
            myid: config.id,
            my_secret_key: match config.crypto_alg {
                crypto::Algorithm::ED25519 => {
                    let mut sk_copy = config.secret_key_bytes.clone();
                    let kp = ed25519::Keypair::decode(&mut sk_copy)
                        .expect("Failed to decode the secret key from the config");
                    Keypair::Ed25519(kp)
                }
                crypto::Algorithm::SECP256K1 => {
                    let sk_copy = config.secret_key_bytes.clone();
                    let sk = secp256k1::SecretKey::from_bytes(sk_copy)
                        .expect("Failed to decode the secret key from the config");
                    let kp = secp256k1::Keypair::from(sk);
                    Keypair::Secp256k1(kp)
                }
                _ => panic!("Unimplemented algorithm"),
            },
            pub_key_map: HashMap::with_capacity(config.num_nodes),
            net_send: net_send,
            cli_send: cli_send,
            storage: Storage::new(EXTRA_SPACE * config.block_size),
            /// The height and next leader are both 1 because the genesis block
            /// is of height 0 and its author is replica 0
            height: 0,
            epoch: 0,
            last_leader: 0,
            last_seen_block: Arc::clone(&genesis_block),
            last_committed_block_ht: 0,
            is_client_apollo_enabled: false,
            payload: config.payload * config.block_size,

            highest_cert: Certificate::empty_cert(),
            highest_height: 0,

            received_propose: None,
            received_propose_sign: None,

            received_vote: Vec::new(),
            received_ack: Vec::new(),

            received_certificate: None,
            received_certificate_sign: None,

            accumulator_pub_params_map: config.bi_pp_map.clone(),
            accumulator_params: config.bi_p.clone().unwrap(),

            received_commit: None,
            received_commit_sign: None,

            propose_share_sent: false,
            vote_cert_share_sent: false,
            commit_share_sent: false,

            propose_gatherer: ShareGatherer::new(config.num_nodes as u16),
            vote_cert_gatherer: ShareGatherer::new(config.num_nodes as u16),
            commit_gatherer: ShareGatherer::new(config.num_nodes as u16),

            rand_beacon_parameter: config.rand_beacon_parameter.clone().unwrap(),
            rand_beacon_queue: config.rand_beacon_queue.clone(),

            reconstruct_queue: std::collections::VecDeque::with_capacity(config.num_nodes * 2),

            shards: vec![std::collections::VecDeque::with_capacity(config.num_nodes); config.num_nodes],
            commits: Vec::with_capacity(config.num_nodes),

            rand_beacon_shares: config.rand_beacon_shares.clone(),
        };
        c.storage
            .committed_blocks_by_hash
            .insert(GENESIS_BLOCK.hash, Arc::clone(&genesis_block));
        c.storage
            .committed_blocks_by_ht
            .insert(0, Arc::clone(&genesis_block));
        for (id, mut pk_data) in &config.pk_map {
            let pk = match config.crypto_alg {
                crypto::Algorithm::ED25519 => {
                    let kp = ed25519::PublicKey::decode(&mut pk_data)
                        .expect("Failed to decode the secret key from the config");
                    PublicKey::Ed25519(kp)
                }
                crypto::Algorithm::SECP256K1 => {
                    let sk = secp256k1::PublicKey::decode(&pk_data)
                        .expect("Failed to decode the secret key from the config");
                    PublicKey::Secp256k1(sk)
                }
                _ => panic!("Unimplemented algorithm"),
            };
            c.pub_key_map.insert(*id, pk);
        }
        c
    }

    pub fn next_leader(&self) -> Replica {
        self.next_of(self.last_leader)
    }

    pub fn next_of(&self, prev: Replica) -> Replica {
        (prev + 1) % self.num_nodes
    }
}
