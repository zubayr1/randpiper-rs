use super::context::Context;
use crypto::*;
use crypto_lib::PublicKey;
use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Deserialize, Serialize};
use types::{DataWithAcc, Replica, SignedData};
use util::io::to_bytes;

pub fn to_shards(data: &[u8], num_nodes: usize, num_faults: usize) -> Vec<Vec<u8>> {
    let num_data_shards = num_nodes - num_faults;
    let shard_size = (data.len() + num_data_shards - 1) / num_data_shards;
    let mut data_with_suffix = data.to_vec();
    let suffix_size = shard_size * num_data_shards - data.len();
    for _ in 0..suffix_size {
        data_with_suffix.push(suffix_size as u8)
    }
    let mut result = Vec::with_capacity(num_nodes);
    for shard in 0..num_data_shards {
        result.push(data_with_suffix[shard * shard_size..(shard + 1) * shard_size].to_vec());
    }
    for _shard in 0..num_faults {
        result.push(vec![0; shard_size]);
    }
    let r = ReedSolomon::new(num_data_shards, num_faults).unwrap();
    r.encode(&mut result).unwrap();
    result
}

pub fn from_shards(mut data: Vec<Option<Vec<u8>>>, num_nodes: usize, num_faults: usize) -> Vec<u8> {
    let num_data_shards = num_nodes - num_faults;
    let r = ReedSolomon::new(num_data_shards, num_faults).unwrap();
    r.reconstruct(&mut data).unwrap();
    let mut result = Vec::with_capacity(num_data_shards * data[0].as_ref().unwrap().len());
    for shard in 0..num_data_shards {
        result.append(&mut data[shard].clone().unwrap());
    }
    result.truncate(result.len() - *result.last().unwrap() as usize);
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn shards() {
        const SIZE: usize = 1024 * 1024;
        let mut array = [0 as u8; SIZE];
        for i in 0..SIZE {
            array[i] = crypto::rand::random();
        }
        let shards = super::to_shards(&array, 4, 1);
        let mut received: Vec<_> = shards.iter().cloned().map(Some).collect();
        received[0] = None;
        let reconstructed = super::from_shards(received, 4, 1);
        assert_eq!(array.to_vec(), reconstructed);
    }
}

pub fn get_size(num_nodes: Replica) -> Replica {
    let mut n: Replica = 1;
    while 1 << n < num_nodes {
        n += 1;
    }
    n + 1
}

pub fn get_acc<T: Serialize>(cx: &Context, data: &T) -> (Vec<Vec<u8>>, DataWithAcc) {
    let shards = to_shards(
        &to_bytes(data),
        cx.num_nodes as usize,
        cx.num_faults as usize,
    );
    let size = get_size(cx.num_nodes) as usize;
    let mut tree = vec![Vec::new(); (1 << size) + 1];
    for i in 0..cx.num_nodes as usize {
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
            sign: cx
                .my_secret_key
                .sign(&hash::ser_and_hash(&tree[1]))
                .unwrap(),
            tree: tree,
            size: size as Replica,
        },
    )
}

pub fn get_sign(acc: &DataWithAcc, n: Replica) -> SignedData {
    let mut vec = Vec::with_capacity(acc.size as usize - 1);
    let mut p = (1 << acc.size - 1 | n) as usize;
    for _ in 0..acc.size - 1 {
        vec.push((acc.tree[p ^ 1].clone(), acc.tree[p >> 1].clone()));
        p >>= 1;
    }
    SignedData {
        sign: acc.sign.clone(),
        start: acc.tree[(1 << acc.size - 1 | n) as usize].clone(),
        index: n,
        chain: vec,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShareGatherer {
    pub size: Replica,
    pub reference: Option<(Vec<u8>, Vec<u8>)>,
    pub shard: Vec<Option<Vec<u8>>>,
    pub shard_num: Replica,
}

impl ShareGatherer {
    pub fn new(num_nodes: Replica) -> Self {
        ShareGatherer {
            size: num_nodes,
            reference: None,
            shard: vec![None; num_nodes as usize],
            shard_num: 0,
        }
    }

    pub fn clear(&mut self) {
        self.reference = None;
        self.shard = vec![None; self.size as usize];
        self.shard_num = 0;
    }

    pub fn add_share(
        &mut self,
        sh: Vec<u8>,
        n: Replica,
        _pp: &EVSSPublicParams381,
        pk: &PublicKey,
        sign: SignedData,
    ) {
        if self.shard[n as usize].is_some() {
            return;
        }
        // The hash should match with the sign.
        if hash::ser_and_hash(&sh).to_vec() != sign.start {
            println!("[WARN] The hash of the shard does not match.");
            debug_assert!(false);
            return;
        }
        if !pk.verify(
            &hash::ser_and_hash(&sign.chain.last().unwrap().1),
            &sign.sign,
        ) {
            println!("[WARN] The signature of the shard does not match.");
            debug_assert!(false);
            return;
        }
        if self.reference.is_none() {
            self.reference = Some((
                hash::ser_and_hash(&sign.chain.last().unwrap().1).to_vec(),
                sign.sign.clone(),
            ));
        } else {
            if self.reference.as_ref().unwrap().1 != sign.sign {
                println!("[WARN] Equivocation detected.");
                debug_assert!(false);
                // TODO: Broadcast the blame.
                return;
            }
        }
        // The share should match with the accumulator.
        let mut change = sign.index;
        change >>= 1;
        for i in 0..sign.chain.len() - 1 {
            let h = if change & 1 == 0 {
                hash::ser_and_hash(&(sign.chain[i].1.clone(), sign.chain[i + 1].0.clone())).to_vec()
            } else {
                hash::ser_and_hash(&(sign.chain[i + 1].0.clone(), sign.chain[i].1.clone())).to_vec()
            };
            if h != sign.chain[i + 1].1 {
                println!("[WARN] Accumulator value does not match.");
                debug_assert!(false);
                return;
            }
            change >>= 1;
        }
        self.shard[n as usize] = Some(sh);
        self.shard_num += 1;
    }

    pub fn reconstruct(&mut self, num_nodes: Replica, num_faults: Replica) -> Option<Vec<u8>> {
        if self.shard_num < num_nodes - num_faults {
            return None;
        }
        Some(from_shards(
            self.shard.clone(),
            num_nodes as usize,
            num_faults as usize,
        ))
    }
}
