use rand::{rngs::StdRng, SeedableRng};
use evss::evss381::*;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SEED: u64 = 42;
const MAX_COUNT: usize = 1000;

pub fn sh_gen(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("sh_gen");
    let secret = F381::rand(rng);
    for n in 3..MAX_COUNT {
        let t = (n + 1) / 2;
        let degree = t - 1;
        let params = EVSS381::setup(secret, degree, rng).expect("");
        let mut shares = Vec::new();
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                for i in 0..n {
                    shares.push(EVSS381::get_share(F381::from((i + 1) as u32), &params, rng).expect(""));
                }
            });
        });
    }
    group.finish();
}

pub fn sh_vrfy(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("sh_vrfy");
    let secret = F381::rand(rng);
    for n in 3..MAX_COUNT {
        let t = (n + 1) / 2;
        let degree = t - 1;
        let params = EVSS381::setup(secret, degree, rng).expect("");
        let mut shares = Vec::new();
        for i in 0..n {
            shares.push(EVSS381::get_share(F381::from((i + 1) as u32), &params, rng).expect(""));
        }
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                for i in 0..n {
                    EVSS381::check(&params.get_public_params(), &shares[i], rng).expect("");
                }
            });
        });
    }
    group.finish();
}

pub fn sh_recon(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("sh_recon");
    let secret = F381::rand(rng);
    for n in 3..MAX_COUNT {
        let t = (n + 1) / 2;
        let degree = t - 1;
        let params = EVSS381::setup(secret, degree, rng).expect("");
        let mut shares = Vec::new();
        for i in 0..n {
            shares.push(EVSS381::get_share(F381::from((i + 1) as u32), &params, rng).expect(""));
        }
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                EVSS381::reconstruct(&shares);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, sh_gen, sh_vrfy, sh_recon);
criterion_main!(benches);
