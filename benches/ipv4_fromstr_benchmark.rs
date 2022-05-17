use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{net::Ipv4Addr, str::FromStr};

#[inline]
fn vmain(s: &str) -> Result<Ipv4Addr, ()> {
    std::net::Ipv4Addr::from_str(s).map_err(|_| ())
}

#[inline]
fn vpatch(s: &str) -> Result<Ipv4Addr, ()> {
    if s.len() < 7 {
        Err(())
    } else {
        std::net::Ipv4Addr::from_str(s).map_err(|_| ())
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Main Parse OK", |b| b.iter(|| vmain(black_box("1.1.1.1"))));
    c.bench_function("Main Parse <7", |b| b.iter(|| vmain(black_box("1.1.1"))));

    c.bench_function("Patch Parse OK", |b| {
        b.iter(|| vpatch(black_box("1.1.1.1")))
    });
    c.bench_function("Patch Parse <7", |b| b.iter(|| vpatch(black_box("1.1.1"))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
