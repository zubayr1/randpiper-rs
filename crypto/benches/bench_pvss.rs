use pvss;

use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};

static TEST_POINTS: [usize; 7] = [
    3, 10, 20, 30, 50, 75, 100
];
const BENCH_COUNT: usize = 10;

pub fn pvss_sh_gen(c: &mut Criterion) {
    let mut group = c.benchmark_group("pvss_sh_gen");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let n = *n;
        let t = (n + 1) / 2;
        let mut keys = Vec::with_capacity(n);
        let mut pubs = Vec::with_capacity(n);
        for _ in 0..n {
            let (public, private) = pvss::crypto::create_keypair();
            keys.push(private);
            pubs.push(public);
        }
        let escrow = pvss::simple::escrow(t as u32);
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                pvss::simple::create_shares(&escrow, &pubs);
            });
        });
    }
    group.finish();
}

pub fn pvss_sh_vrfy(c: &mut Criterion) {
    let mut group = c.benchmark_group("pvss_sh_vrfy");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let n = *n;
        let t = (n + 1) / 2;
        let mut keys = Vec::with_capacity(n);
        let mut pubs = Vec::with_capacity(n);
        for _ in 0..n {
            let (public, private) = pvss::crypto::create_keypair();
            keys.push(private);
            pubs.push(public);
        }
        let escrow = pvss::simple::escrow(t as u32);
        let commitments = pvss::simple::commitments(&escrow);
        let shares = pvss::simple::create_shares(&escrow, &pubs);
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                for share in &shares {
                    let idx = (share.id - 1) as usize;
                    share.verify(share.id, &pubs[idx], &escrow.extra_generator, &commitments);
                    let d = pvss::simple::decrypt_share(&keys[idx], &pubs[idx], &share);
                    d.verify(&pubs[idx], &share);
                }
            })
        });
    }
    group.finish();
}

pub fn pvss_sh_recon(c: &mut Criterion) {
    let mut group = c.benchmark_group("pvss_sh_recon");
    BenchmarkGroup::sampling_mode(&mut group, criterion::SamplingMode::Flat);
    for n in &TEST_POINTS {
        let n = *n;
        let t = (n + 1) / 2;
        let mut keys = Vec::with_capacity(n);
        let mut pubs = Vec::with_capacity(n);
        for _ in 0..n {
            let (public, private) = pvss::crypto::create_keypair();
            keys.push(private);
            pubs.push(public);
        }
        let escrow = pvss::simple::escrow(t as u32);
        let mut decrypted = Vec::with_capacity(n);
        let shares = pvss::simple::create_shares(&escrow, &pubs);
        for share in &shares {
            let idx = (share.id - 1) as usize;
            let d = pvss::simple::decrypt_share(&keys[idx], &pubs[idx], &share);
            decrypted.push(d);
        }
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                pvss::simple::recover(t as u32, decrypted.as_slice()).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(BENCH_COUNT);
    targets = pvss_sh_gen, pvss_sh_vrfy, pvss_sh_recon);
criterion_main!(benches);
