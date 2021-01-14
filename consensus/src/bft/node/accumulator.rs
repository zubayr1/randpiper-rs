use super::context::Context;
use crypto::*;
use crypto::rand::{SeedableRng, rngs::StdRng};
use types::{DataWithAcc, SignedData, Replica};
use util::io::to_bytes;
use serde::{Deserialize, Serialize};
use reed_solomon_erasure::galois_8::ReedSolomon;

pub fn to_shards(data: &[u8], num_nodes: usize, num_faults: usize) -> Vec<Vec<u8>> {
    let num_data_shards = num_nodes - num_faults;
    let shard_size = (data.len() + num_data_shards - 1) / num_data_shards;
    let mut data_with_suffix = data.to_vec();
    let suffix_size = shard_size * num_data_shards - data.len();
    for _ in 0..suffix_size {
        data_with_suffix.push(suffix_size as u8)
    }
    let mut result = Vec::new();
    for shard in 0..num_data_shards {
        result.push(data_with_suffix[shard * shard_size..(shard + 1) * shard_size].to_vec());
    }
    for shard in 0..num_faults {
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
    let mut result = Vec::new();
    for shard in 0..num_data_shards {
        result.append(&mut data[shard].clone().unwrap());
    }
    result.truncate(result.len() - *result.last().unwrap() as usize);
    result
}

pub fn get_acc<T: Serialize>(cx: &Context, data: &T) -> DataWithAcc {
    let shards = to_shards(&to_bytes(data), cx.num_nodes as usize, cx.num_faults as usize);
    let mut values = Vec::new();
    for shard in shards.iter() {
        values.push(F381::from_be_bytes_mod_order(&hash::ser_and_hash(&shard)));
    }
    let poly = Biaccumulator381::commit(&cx.accumulator_params, &values[..], &mut StdRng::from_entropy()).unwrap();
    let mut acc = Vec::new();
    for value in values.iter() {
        acc.push(Biaccumulator381::create_witness(*value, &cx.accumulator_params, &poly, &mut StdRng::from_entropy()).unwrap());
    }
    DataWithAcc {
        hash: hash::ser_and_hash(data).to_vec(),
        shares: acc,
    }
}

pub fn get_sign<T: Serialize>(cx: &Context, data: &T) -> SignedData {
    let data_with_acc = get_acc(cx, data);
    SignedData {
        sign: cx.my_secret_key.sign(&hash::ser_and_hash(&data_with_acc)).unwrap(),
        data: data_with_acc,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShareGatherer {
    pub size: Replica,
    pub reference: Option<SignedData>,
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
        self.shard = vec![None; self.size as usize];
        self.shard_num = 0;
    }

    pub fn add_share(&mut self, sh: Vec<u8>, n: Replica, sign: SignedData) {
        //TODO: Check the sign.
        self.shard[n as usize] = Some(sh);
        self.shard_num += 1;
    }

    pub fn reconstruct(&mut self, num_nodes: Replica, num_faults: Replica) -> Option<Vec<u8>> {
        if self.shard_num < num_nodes - num_faults {
            return None;
        }
        Some(from_shards(self.shard.clone(), num_nodes as usize, num_faults as usize))
    }

}
