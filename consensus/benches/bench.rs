extern crate consensus;
use consensus::bft::node::accumulator;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use crypto::rand::{rngs::StdRng, SeedableRng};
use crypto::*;
use std::collections::HashMap;
use types::{Block, Certificate, Content, Propose, Vote};
use util::io::to_bytes;

const SEED: u64 = 42;
static TEST_POINTS: [usize; 7] = [3, 10, 20, 30, 50, 75, 100];
const BENCH_COUNT: usize = 10;

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

pub fn propose_to_shards(c: &mut Criterion) {
    let propose_map = generate_propose();
    let mut group = c.benchmark_group("propose_to_shards");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let data = propose_map.get(&n).unwrap();
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &n| {
            b.iter(|| accumulator::to_shards(&to_bytes(&data), n, (n - 1) / 2))
        });
    }
    group.finish();
}

pub fn shards_to_propose(c: &mut Criterion) {
    let propose_map = generate_propose();
    let mut group = c.benchmark_group("shards_to_propose");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let data = propose_map.get(n).unwrap();
        let shards = accumulator::to_shards(&to_bytes(&data), *n, (*n - 1) / 2);
        let mut received: Vec<_> = shards.iter().cloned().map(Some).collect();
        for i in 0..(n - 1) / 2 {
            received[i] = None;
        }
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &n| {
            b.iter_batched(
                || received.clone(),
                |d| accumulator::from_shards(d, n, (n - 1) / 2),
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(BENCH_COUNT);
    targets = propose_to_shards, shards_to_propose);
criterion_main!(benches);
