#![feature(test)]

extern crate rand;
extern crate splittable_random as splittable;
extern crate test;

const RAND_BENCH_N: u64 = 1000;

use std::mem::size_of;
use test::{black_box, Bencher};
use rand::{Rng, OsRng};
use splittable::siprng::SipRng;

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
