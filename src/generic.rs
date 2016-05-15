//! A construction that turns a pair of a splittable and a sequential
//! PRNG into a splittable PRNG.  The intent of this is to gain the
//! splittability of the former but retain the sequential generation
//! speed of the latter.
//!
//! No claims are made that the construct generically inherits any
//! good qualities of its component PRNGs.  In particular, **don't
//! assume that the composition of two secure PRNGs is also secure**.

use rand::{Rng, SeedableRng, Rand};
use super::{SplitRng, SplitPrf};
use std::marker::PhantomData;


/// Wrapper that turns a `SplitRng` `S` and an `Rng` `R` into a `SplitRng`.
pub struct Split<S, R> {
    rng: S,
    seq: R
}

/// The PRF type that corresponds to `Split`.
pub struct Prf<F, R> {
    prf: F,
    seq: PhantomData<R>
}


impl<S: SplitRng, R: Rng> Rng for Split<S, R> {

    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.seq.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.seq.next_u64()
    }

    #[inline(always)]
    fn next_f32(&mut self) -> f32 {
        self.seq.next_f32()
    }

    #[inline(always)]
    fn next_f64(&mut self) -> f64 {
        self.seq.next_f64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.seq.fill_bytes(dest);        
    }

}

impl<S, SeedS, R> SeedableRng<SeedS> for Split<S, R> 
    where S: SplitRng + SeedableRng<SeedS>, 
          R: Rng + Rand
{
    fn reseed(&mut self, seed: SeedS) {
        self.rng.reseed(seed);
        self.seq = self.rng.gen();
    }

    fn from_seed(seed: SeedS) -> Self {
        let mut rng = S::from_seed(seed);
        let seq = rng.gen();
        Split {
            rng: rng, 
            seq: seq
        }
    }
}


impl<S, R> SplitRng for Split<S, R> 
    where S: SplitRng,
          R: Rng + Rand
{
    type Prf = Prf<S::Prf, R>;
    
    fn split(&mut self) -> Self {
        let mut rng = self.rng.split();
        let seq = rng.gen();
        Split {
            rng: rng,
            seq: seq
        }
    }

    fn splitn(&mut self) -> Self::Prf {
        Prf {
            prf: self.rng.splitn(),
            seq: PhantomData
        }
    }
}

impl<S, F, R> SplitPrf<Split<S, R>> for Prf<F, R> 
    where S: SplitRng,
          F: SplitPrf<S>,
          R: Rand
{
    fn call(&self, i: u64) -> Split<S, R> {
        let mut rng = self.prf.call(i);
        let seq = rng.gen();
        Split {
            rng: rng,
            seq: seq
        }
    }
    
}

impl<S: Rng + Rand, R: Rand> Rand for Split<S, R> {
    fn rand<G: Rng>(other: &mut G) -> Self {
        let mut rng: S = other.gen();
        let seq: R = rng.gen();
        Split {
            rng: rng,
            seq: seq
        }
    }
}
