use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_ipv4_optimization::Parser;
use std::{net::Ipv4Addr, str::FromStr};

/// <https://url.spec.whatwg.org/#ipv4-number-parser>
#[inline]
fn parse_ipv4number(mut input: &str) -> Result<Option<u32>, ()> {
    let mut r = 10;
    if input.starts_with("0x") || input.starts_with("0X") {
        input = &input[2..];
        r = 16;
    } else if input.len() >= 2 && input.starts_with('0') {
        input = &input[1..];
        r = 8;
    }

    // At the moment we can't know the reason why from_str_radix fails
    // https://github.com/rust-lang/rust/issues/22639
    // So instead we check if the input looks like a real number and only return
    // an error when it's an overflow.
    let valid_number = match r {
        8 => input.chars().all(|c| ('0'..='7').contains(&c)),
        10 => input.chars().all(|c| ('0'..='9').contains(&c)),
        16 => input.chars().all(|c| {
            ('0'..='9').contains(&c) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
        }),
        _ => false,
    };

    if !valid_number {
        return Ok(None);
    }

    if input.is_empty() {
        return Ok(Some(0));
    }
    if input.starts_with('+') {
        return Ok(None);
    }
    match u32::from_str_radix(input, r) {
        Ok(number) => Ok(Some(number)),
        Err(_) => Err(()),
    }
}

#[inline]
fn vurl(input: &str) -> Result<Ipv4Addr, ()> {
    if input.is_empty() {
        return Err(());
    }
    let mut parts: Vec<&str> = input.split('.').collect();
    if parts.last() == Some(&"") {
        parts.pop();
    }
    if parts.len() > 4 {
        return Err(());
    }
    let mut numbers: Vec<u32> = Vec::new();
    let mut overflow = false;
    for part in parts {
        if part.is_empty() {
            return Err(());
        }
        match parse_ipv4number(part) {
            Ok(Some(n)) => numbers.push(n),
            Ok(None) => return Err(()),
            Err(()) => overflow = true,
        };
    }
    if overflow {
        return Err(());
    }
    let mut ipv4 = numbers.pop().expect("a non-empty list of numbers");
    // Equivalent to: ipv4 >= 256 ** (4 − numbers.len())
    if ipv4 > u32::max_value() >> (8 * numbers.len() as u32) {
        return Err(());
    }
    if numbers.iter().any(|x| *x > 255) {
        return Err(());
    }
    for (counter, n) in numbers.iter().enumerate() {
        ipv4 += n << (8 * (3 - counter as u32))
    }
    Ok(Ipv4Addr::from(ipv4))
}

#[inline]
fn vmain(s: &str) -> Result<Ipv4Addr, ()> {
    std::net::Ipv4Addr::from_str(s).map_err(|_| ())
}

#[inline]
fn vpatch(s: &str) -> Result<Ipv4Addr, ()> {
    Parser::new(s)
        .parse_with(
            |p| p.read_ipv4_addr(),
            rust_ipv4_optimization::AddrKind::Ipv4,
        )
        .map_err(|_| ())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Main Parse OK", |b| b.iter(|| vmain(black_box("1.1.1.1"))));
    c.bench_function("Patch Parse OK", |b| {
        b.iter(|| vpatch(black_box("1.1.1.1")))
    });
    c.bench_function("URL Parse OK", |b| b.iter(|| vurl(black_box("1.1.1.1"))));

    c.bench_function("Main Parse <7", |b| b.iter(|| vmain(black_box("1.1.1"))));
    c.bench_function("Patch Parse <7", |b| b.iter(|| vpatch(black_box("1.1.1"))));
    c.bench_function("URL Parse <7", |b| b.iter(|| vurl(black_box("1.1.1"))));

    c.bench_function("Main Parse empty", |b| b.iter(|| vmain(black_box(""))));
    c.bench_function("Patch Parse empty", |b| b.iter(|| vpatch(black_box(""))));
    c.bench_function("URL Parse empty", |b| b.iter(|| vurl(black_box(""))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
