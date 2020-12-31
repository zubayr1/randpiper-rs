use rand::{rngs::StdRng, SeedableRng};
use evss::biaccumulator381::*;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SEED: u64 = 42;
const MAX_COUNT: usize = 1000;

pub fn bi_sh_gen(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("bi_sh_gen");
    for n in 3..MAX_COUNT {
        let vec: Vec<F381> = (0..n).map(|_| F381::rand(rng)).collect();
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let params = Biaccumulator381::setup(&vec[..], n, rng).expect("");
                for cred in &vec {
                    Biaccumulator381::create_witness(*cred, &params, rng).expect("");
                }
            });
        });
    }
    group.finish();
}

pub fn bi_sh_vrfy(c: &mut Criterion) {
    let rng = &mut StdRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("bi_sh_vrfy");
    for n in 3..MAX_COUNT {
        let vec: Vec<F381> = (0..n).map(|_| F381::rand(rng)).collect();
        let params = Biaccumulator381::setup(&vec[..], n, rng).expect("");
        let mut shares = Vec::new();
        for cred in &vec {
           shares.push(Biaccumulator381::create_witness(*cred, &params, rng).expect(""));
        }
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                for wit in &shares {
                    Biaccumulator381::check(&params.get_public_params(), wit, rng).expect("");
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bi_sh_gen, bi_sh_vrfy);
criterion_main!(benches);
