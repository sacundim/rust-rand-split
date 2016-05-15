#![feature(test)]

extern crate mersenne_twister;
extern crate rand;
extern crate rand_split;
extern crate test;

const RAND_BENCH_N: u64 = 1000;

use mersenne_twister::{MT19937, MT19937_64};
use rand::{Rng, SeedableRng, OsRng, StdRng, XorShiftRng};
use rand::chacha::ChaChaRng;
use rand::isaac::{IsaacRng, Isaac64Rng};
use rand_split::Split;
use rand_split::siprng::SipRng;
use std::mem::size_of;
use test::{black_box, Bencher};

#[bench]
fn rand_siprng(b: &mut Bencher) {
    let mut rng: SipRng = OsRng::new().unwrap().gen();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}

#[bench]
fn rand_split_isaac64(b: &mut Bencher) {
    let mut rng: Split<Isaac64Rng> = OsRng::new().unwrap().gen();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}

/*
 * The following benches are lifted straight from the `rand` crate.
 * Having them here is just convenient.
 */

#[bench]
fn rand_xorshift(b: &mut Bencher) {
    let mut rng: XorShiftRng = OsRng::new().unwrap().gen();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}

#[bench]
fn rand_isaac(b: &mut Bencher) {
    let mut rng: IsaacRng = OsRng::new().unwrap().gen();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}

#[bench]
fn rand_isaac64(b: &mut Bencher) {
    let mut rng: Isaac64Rng = OsRng::new().unwrap().gen();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}

#[bench]
fn rand_std(b: &mut Bencher) {
    let mut rng = StdRng::new().unwrap();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}


/*
 * The `rand` crate is missing a benchmark for `ChaChaRng`.
 */

#[bench]
fn rand_chacha(b: &mut Bencher) {
    let mut rng: ChaChaRng = OsRng::new().unwrap().gen();
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}


/*
 * Benchmarks to compare to the `mersenne_twister` crate.
 */

#[bench]
fn rand_mt19937(b: &mut Bencher) {
    let seed: u64 = OsRng::new().unwrap().gen();
    let mut rng: MT19937 = SeedableRng::from_seed(seed);
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}

#[bench]
fn rand_mt19937_64(b: &mut Bencher) {
    let seed: u64 = OsRng::new().unwrap().gen();
    let mut rng: MT19937_64 = SeedableRng::from_seed(seed);
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}


