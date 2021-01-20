extern crate consensus;
use consensus::bft::node::accumulator;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use crypto::rand::{rngs::StdRng, SeedableRng};
use crypto::*;
use serde::Serialize;
use std::collections::HashMap;
use types::{Block, Certificate, Content, DataWithAcc, Propose, Replica, SignedData, Vote};
use util::io::to_bytes;

const SEED: u64 = 42;
static TEST_POINTS: [usize; 7] = [3, 10, 20, 30, 50, 75, 100];
const BENCH_COUNT: usize = 10;

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

fn tree_check_dummy_share(sign: SignedData) {
    let mut change = sign.index;
    change >>= 1;
    for i in 0..sign.chain.len() - 1 {
        if change & 1 == 0 {
            hash::ser_and_hash(&(sign.chain[i].1.clone(), sign.chain[i + 1].0.clone())).to_vec();
        } else {
            hash::ser_and_hash(&(sign.chain[i + 1].0.clone(), sign.chain[i].1.clone())).to_vec();
        }
        change >>= 1;
    }
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

pub fn tree_propose_to_shards(c: &mut Criterion) {
    let propose_map = generate_propose();
    let mut group = c.benchmark_group("tree_propose_to_shards");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let data = propose_map.get(&n).unwrap();
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &n| {
            b.iter(|| tree_get_dummy_acc(n as u16, ((n - 1) / 2) as u16, &data))
        });
    }
    group.finish();
}

pub fn tree_shards_to_propose(c: &mut Criterion) {
    let propose_map = generate_propose();
    let mut group = c.benchmark_group("tree_shards_to_propose");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let data = propose_map.get(n).unwrap();
        let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, &data);
        let shards = acc.0.clone();
        let mut received: Vec<_> = shards.iter().cloned().map(Some).collect();
        for i in 0..(n - 1) / 2 {
            received[i] = None;
        }
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &n| {
            b.iter_batched(
                || received.clone(),
                |d| {
                    for i in 0..n {
                        tree_check_dummy_share(accumulator::get_sign(&acc.1, i as Replica));
                    }
                    accumulator::from_shards(d, n, (n - 1) / 2);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

pub fn bi_propose_to_shards(c: &mut Criterion) {
    let propose_map = generate_propose();
    let mut group = c.benchmark_group("bi_propose_to_shards");
    let rng = &mut rand::rngs::StdRng::from_entropy();
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let data = propose_map.get(&n).unwrap();
        let params = crypto::Biaccumulator381::setup(*n, rng).unwrap();
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &n| {
            b.iter(|| {
                let shards = accumulator::to_shards(&to_bytes(&data), n, (n - 1) / 2);
                let mut values = Vec::with_capacity(n as usize);
                for shard in shards.iter() {
                    values.push(F381::from_be_bytes_mod_order(&hash::ser_and_hash(&shard)));
                }
                let poly =
                    Biaccumulator381::commit(&params, &values[..], &mut StdRng::from_entropy())
                        .unwrap();
                let mut acc = Vec::with_capacity(n as usize);
                for value in values.iter() {
                    acc.push(
                        Biaccumulator381::create_witness(
                            *value,
                            &params,
                            &poly,
                            &mut StdRng::from_entropy(),
                        )
                        .unwrap(),
                    );
                }
            });
        });
    }
    group.finish();
}

pub fn bi_shards_to_propose(c: &mut Criterion) {
    let propose_map = generate_propose();
    let mut group = c.benchmark_group("bi_shards_to_propose");
    let rng = &mut rand::rngs::StdRng::from_entropy();
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let data = propose_map.get(&n).unwrap();
        let params = crypto::Biaccumulator381::setup(*n, rng).unwrap();
        let acc = tree_get_dummy_acc(*n as u16, ((n - 1) / 2) as u16, &data);
        let shards = acc.0.clone();
        let mut received: Vec<_> = shards.iter().cloned().map(Some).collect();
        for i in 0..(n - 1) / 2 {
            received[i] = None;
        }
        let mut values = Vec::with_capacity(*n as usize);
        for shard in shards.iter() {
            values.push(F381::from_be_bytes_mod_order(&hash::ser_and_hash(&shard)));
        }
        let poly =
            Biaccumulator381::commit(&params, &values[..], &mut StdRng::from_entropy()).unwrap();
        let mut acc = Vec::with_capacity(*n as usize);
        for value in values.iter() {
            acc.push(
                Biaccumulator381::create_witness(
                    *value,
                    &params,
                    &poly,
                    &mut StdRng::from_entropy(),
                )
                .unwrap(),
            );
        }
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &n| {
            b.iter_batched(
                || received.clone(),
                |d| {
                    for wit in acc.iter() {
                        crypto::EVSS381::check(
                            &params.get_public_params(),
                            &poly.get_commit(),
                            &wit,
                            rng,
                        )
                        .unwrap();
                    }
                    accumulator::from_shards(d, n, (n - 1) / 2);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(BENCH_COUNT);
    targets = tree_propose_to_shards, tree_shards_to_propose, bi_propose_to_shards, bi_shards_to_propose);
criterion_main!(benches);
