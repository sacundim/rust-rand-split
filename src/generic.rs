//! A construction that turns a pair of a splittable and a sequential
//! PRNG into a splittable PRNG.  The intent of this is to gain the
//! splittability of the former but retain the sequential generation
//! speed of the latter.
//!
//! No claims are made that the construct generically inherits any
//! good qualities of its component PRNGs.  In particular, **don't
//! assume that the composition of two secure PRNGs is also secure**.

use rand::{Rng, SeedableRng, Rand};
use super::{SplitRng, RngBranch};
use std::marker::PhantomData;


/// Wrapper that turns a `SplitRng` `S` and an `Rng` `R` into a `SplitRng`.
pub struct Split<S, R> {
    splitter: S,
    sequential: R
}

/// A branch of a `Split`.
pub struct Branch<S, R> {
    branch: S,
    sequential: PhantomData<R>
}


impl<S: SplitRng, R: Rng> Rng for Split<S, R> {

    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.sequential.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.sequential.next_u64()
    }

    #[inline(always)]
    fn next_f32(&mut self) -> f32 {
        self.sequential.next_f32()
    }

    #[inline(always)]
    fn next_f64(&mut self) -> f64 {
        self.sequential.next_f64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.sequential.fill_bytes(dest);        
    }

}

impl<S, SeedS, R> SeedableRng<SeedS> for Split<S, R> 
    where S: SplitRng + SeedableRng<SeedS>, 
          R: Rng + Rand
{
    fn reseed(&mut self, seed: SeedS) {
        self.splitter.reseed(seed);
        self.sequential = self.splitter.gen();
    }

    fn from_seed(seed: SeedS) -> Self {
        let mut splitter = S::from_seed(seed);
        let sequential = splitter.gen();
        Split {
            splitter: splitter, 
            sequential: sequential
        }
    }
}


impl<S, R> SplitRng for Split<S, R> 
    where S: SplitRng,
          R: Rng + Rand
{
    type Branch = Branch<S::Branch, R>;
    
    fn splitn(self) -> Self::Branch {
        Branch {
            branch: self.splitter.splitn(),
            sequential: PhantomData
        }
    }
}

impl<S, B, R> RngBranch<Split<S, R>> for Branch<B, R> 
    where S: SplitRng,
          B: RngBranch<S>,
          R: Rand
{
    fn branch(&self, i: usize) -> Split<S, R> {
        let mut splitter = self.branch.branch(i);
        let sequential = splitter.gen();
        Split {
            splitter: splitter,
            sequential: sequential
        }
    }
    
}

impl<S: Rng + Rand, R: Rand> Rand for Split<S, R> {
    fn rand<G: Rng>(rng: &mut G) -> Self {
        let mut splitter: S = rng.gen();
        let sequential: R = splitter.gen();
        Split {
            splitter: splitter,
            sequential: sequential
        }
    }
}
