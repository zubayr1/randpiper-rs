use rand::{rngs::StdRng, SeedableRng};
use evss::evss381::*;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput, BenchmarkGroup};

const SEED: u64 = 42;
const TEST_POINTS: [usize; 19] = [3, 10, 30, 60, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 700, 800, 900, 1000];
const BENCH_COUNT:usize = 10;

pub fn evss_sh_gen(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("evss_sh_gen");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    let secret = F381::rand(rng);
    for n in &TEST_POINTS {
        let t = (*n + 1) / 2;
        let degree = t - 1;
        let params = EVSS381::setup(secret, degree, rng).expect("");
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                for i in 0..*n {
                    EVSS381::get_share(F381::from((i + 1) as u32), &params, rng).expect("");
                }
            });
        });
    }
    group.finish();
}

pub fn evss_sh_vrfy(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("evss_sh_vrfy");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    let secret = F381::rand(rng);
    for n in &TEST_POINTS {
        let t = (*n + 1) / 2;
        let degree = t - 1;
        let params = EVSS381::setup(secret, degree, rng).expect("");
        let mut shares = Vec::new();
        for i in 0..*n {
            shares.push(EVSS381::get_share(F381::from((i + 1) as u32), &params, rng).expect(""));
        }
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                for i in 0..*n {
                    EVSS381::check(&params.get_public_params(), &shares[i], rng).expect("");
                }
            });
        });
    }
    group.finish();
}

pub fn evss_sh_recon(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("evss_sh_recon");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    let secret = F381::rand(rng);
    for n in &TEST_POINTS {
        let t = (*n + 1) / 2;
        let degree = t - 1;
        let params = EVSS381::setup(secret, degree, rng).expect("");
        let mut shares = Vec::new();
        for i in 0..*n {
            shares.push(EVSS381::get_share(F381::from((i + 1) as u32), &params, rng).expect(""));
        }
        group.throughput(Throughput::Bytes(*n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                EVSS381::reconstruct(&shares);
            });
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(BENCH_COUNT);
    targets = evss_sh_gen, evss_sh_vrfy, evss_sh_recon);
criterion_main!(benches);
