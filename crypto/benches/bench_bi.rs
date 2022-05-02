use evss::biaccumulator381::*;
use rand::{rngs::StdRng, SeedableRng};

use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};

const SEED: u64 = 42;
static TEST_POINTS: [usize; 7] = [
    3, 10, 20, 30, 50, 75, 100
];
const BENCH_COUNT: usize = 10;

pub fn bi_sh_gen(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("bi_sh_gen");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let vec: Vec<F381> = (0..*n).map(|_| F381::rand(rng)).collect();
        let params = Biaccumulator381::setup(*n, rng).expect("");
        let poly = Biaccumulator381::commit(&params, &vec[..], rng).expect("");
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(*n), n, |b, &_n| {
            b.iter(|| {
                for cred in &vec {
                    Biaccumulator381::create_witness(*cred, &params, &poly, rng).expect("");
                }
            });
        });
    }
    group.finish();
}

pub fn bi_sh_vrfy(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("bi_sh_vrfy");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let vec: Vec<F381> = (0..*n).map(|_| F381::rand(rng)).collect();
        let params = Biaccumulator381::setup(*n, rng).expect("");
        let poly = Biaccumulator381::commit(&params, &vec[..], rng).expect("");
        let mut shares = Vec::new();
        for cred in &vec {
            shares.push(Biaccumulator381::create_witness(*cred, &params, &poly, rng).expect(""));
        }
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                for wit in &shares {
                    Biaccumulator381::check(&params.get_public_params(), &poly.get_commit(), wit, rng).expect("");
                }
            });
        });
    }
    group.finish();
}

// criterion_group!(benches, bi_sh_gen, bi_sh_vrfy);
criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(BENCH_COUNT);
    targets = bi_sh_gen, bi_sh_vrfy);
criterion_main!(benches);
