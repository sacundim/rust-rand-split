#![feature(test)]

extern crate mersenne_twister;
extern crate pcg_rand as pcg;
extern crate rand;
extern crate rand_split;
extern crate test;

const RAND_BENCH_N: u64 = 1000;

use mersenne_twister::{MT19937, MT19937_64};
use pcg::{Pcg32, Pcg32Fast, Pcg32L, Pcg32LFast, Pcg64, Pcg64Fast};
use rand::{Rng, SeedableRng, Rand, thread_rng, StdRng, XorShiftRng};
use rand::chacha::ChaChaRng;
use rand::isaac::{IsaacRng, Isaac64Rng};
use rand_split::Split;
use rand_split::siprng::SipRng;
use std::mem::size_of;
use test::{black_box, Bencher};

// Adapted from the `rand` crate.
fn bench_random_rng<R: Rng + Rand>(b: &mut Bencher) {
    bench_rng::<R>(b, &mut thread_rng().gen());
}

fn bench_rng<R: Rng>(b: &mut Bencher, rng: &mut R) {
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            black_box(rng.gen::<usize>());
        }
    });
    b.bytes = size_of::<usize>() as u64 * RAND_BENCH_N;
}


#[bench]
fn rand_siprng(b: &mut Bencher) {
    bench_random_rng::<SipRng>(b);
}

#[bench]
fn rand_split_sip_isaac64(b: &mut Bencher) {
    bench_random_rng::<Split<Isaac64Rng>>(b);
}

/*
 * The following benches are lifted straight from the `rand` crate.
 * Having them here is just convenient.
 */

#[bench]
fn rand_xorshift(b: &mut Bencher) {
    bench_random_rng::<XorShiftRng>(b);
}

#[bench]
fn rand_isaac(b: &mut Bencher) {
    bench_random_rng::<IsaacRng>(b);
}

#[bench]
fn rand_isaac64(b: &mut Bencher) {
    bench_random_rng::<Isaac64Rng>(b);
}

#[bench]
fn rand_std(b: &mut Bencher) {
    bench_rng::<StdRng>(b, &mut StdRng::new().unwrap());
}


/*
 * The `rand` crate is missing a benchmark for `ChaChaRng`.
 */

#[bench]
fn rand_chacha(b: &mut Bencher) {
    bench_random_rng::<ChaChaRng>(b);
}


/*
 * Benchmarks to compare to the `mersenne_twister` crate.
 */

#[bench]
fn rand_mt19937(b: &mut Bencher) {
    let seed: u64 = thread_rng().gen();
    bench_rng::<MT19937>(b, &mut SeedableRng::from_seed(seed));
}

#[bench]
fn rand_mt19937_64(b: &mut Bencher) {
    let seed: u64 = thread_rng().gen();
    bench_rng::<MT19937_64>(b, &mut SeedableRng::from_seed(seed));
}

/*
 * pcg_rand crate
 */

#[bench]
fn rand_pgc32(b: &mut Bencher) {
    bench_random_rng::<Pcg32>(b);
}

#[bench]
fn rand_pgc32_fast(b: &mut Bencher) {
    bench_random_rng::<Pcg32Fast>(b);
}

#[bench]
fn rand_pgc32l(b: &mut Bencher) {
    bench_random_rng::<Pcg32L>(b);
}

#[bench]
fn rand_pgc32l_fast(b: &mut Bencher) {
    bench_random_rng::<Pcg32LFast>(b);
}

#[bench]
fn rand_pgc64(b: &mut Bencher) {
    bench_random_rng::<Pcg64>(b);
}

#[bench]
fn rand_pgc64_fast(b: &mut Bencher) {
    bench_random_rng::<Pcg64Fast>(b);
}

