//! A splittable pseudo-random number generator based on the
//! SipHash function.  **This is not intended to be a
//! cryptographically secure PRNG.**
//!
//! This generator is broadly modeled after Claessen and Pałka's, a
//! version of which is implemented in the Haskell [`tf-random`
//! library](https://hackage.haskell.org/package/tf-random).  Instead
//! of the Skein hash function, however, we use SipHash as the
//! pseudo-random function.
//!
//!
//! ## References
//!
//! * Aumasson, Jean-Philippe and Daniel J. Bernstein.  2012.
//!   ["SipHash: a fast short-input
//!   PRF."](https://eprint.iacr.org/2012/351) Cryptology ePrint
//!   Archive, Report 2012/351.
//! * Claessen, Koen and Michał H. Pałka.  2013.  ["Splittable
//!   Pseudorandom Number Generators using Cryptographic
//!   Hashing."](http://publications.lib.chalmers.se/records/fulltext/183348/local_183348.pdf)
//!   *Haskell '13: Proceedings of the 2013 ACM SIGPLAN symposium on
//!   Haskell*, pp. 47-58.

use rand::{Rand, Rng, SeedableRng};
use super::{SplitRng, SplitPrf};
use std::mem;


/// A splittable pseudorandom generator based on SipHash.
pub struct SipRng {
    v0:  u64,
    v1:  u64,
    v2:  u64,
    v3:  u64,
    ctr: u64,
    len: usize
}

/// A PRF taken off a `SipRng`.
pub struct SipPrf(SipRng);


macro_rules! sipround {
    ($v0: expr, $v1: expr, $v2: expr, $v3: expr) => {
        $v0 = $v0.wrapping_add($v1); $v2 = $v2.wrapping_add($v3);
        $v1 = $v1.rotate_left(13);   $v3 = $v3.rotate_left(16);
        $v1 ^= $v0;                  $v3 ^= $v2;
        $v0 = $v0.rotate_left(32);
        
        $v2 = $v2.wrapping_add($v1); $v0 = $v0.wrapping_add($v3);
        $v1 = $v1.rotate_left(17);   $v3 = $v3.rotate_left(21);
        $v1 ^= $v2;                  $v3 ^= $v0;
        $v2 = $v0.rotate_left(32);
    }
}

const C0: u64 = 0x736f6d6570736575;
const C1: u64 = 0x646f72616e646f6d;
const C2: u64 = 0x6c7967656e657261;
const C3: u64 = 0x7465646279746573;

impl SipRng {
    /// Create a `SipRng` generator from two `u64`s given as seed.
    pub fn new(k0: u64, k1: u64) -> SipRng {
        SipRng { 
            v0:  k0 ^ C0,
            v1:  k1 ^ C1,
            v2:  k0 ^ C2,
            v3:  k1 ^ C3,
            ctr: 0,
            len: 1
        }
    }

    fn clone(&self) -> SipRng {
        SipRng { 
            v0:  self.v0,
            v1:  self.v1,
            v2:  self.v2,
            v3:  self.v3,
            ctr: self.ctr,
            len: self.len
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.v3 ^= self.ctr;
        sipround!(self.v0, self.v1, self.v2, self.v3);
        self.v0 ^= self.ctr;
        self.ctr = self.ctr.wrapping_add(1);
        if self.ctr == 0 {
            self.descend(0);
        }
    }

    #[inline]
    fn descend(&mut self, i: u64) {
        self.v3 ^= i;
        sipround!(self.v0, self.v1, self.v2, self.v3);
        self.v0 ^= i;
        self.len = self.len.wrapping_add(1);
        self.ctr = 0;
    }

}

impl SplitPrf<SipRng> for SipPrf {
    fn call(&self, i: u64) -> SipRng {
        let mut r = self.0.clone();
        r.descend(i as u64);
        r
    }
}

impl SplitRng for SipRng {
    type Prf = SipPrf;

    fn split(&mut self) -> Self {
        let mut child = self.clone();
        self.descend(0);
        child.descend(1);
        child
    }

    fn splitn(&mut self) -> SipPrf {
        let child = self.split();
        SipPrf(child)
    }

}

impl Rng for SipRng {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.advance();
        let (mut v0, mut v1, mut v2, mut v3) = 
            (self.v0, self.v1, self.v2, self.v3);

        let len = (self.len as u64).wrapping_shl(56);
        v3 ^= len;
        sipround!(v0, v1, v2, v3);
        v0 ^= len;

        v2 ^= 0xff;
        sipround!(v0, v1, v2, v3);
        sipround!(v0, v1, v2, v3);
        sipround!(v0, v1, v2, v3);
        v0 ^ v1 ^ v2 ^ v3
    }
    
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let block = unsafe {
                mem::transmute::<u64, [u8; 8]>(self.next_u64())
            };
            for i in 0..chunk.len() {
                chunk[i] = block[i];
            }
        }
    }
}

impl SeedableRng<(u64, u64)> for SipRng {
    
    fn reseed(&mut self, seed: (u64, u64)) {
        self.v0 = seed.0 ^ C0;
        self.v1 = seed.1 ^ C1;
        self.v2 = seed.0 ^ C2;
        self.v3 = seed.1 ^ C3;
        self.len = 1;
        self.ctr = 0;
    }
    
    fn from_seed(seed: (u64, u64)) -> SipRng {
        let (k0, k1) = seed;
        SipRng::new(k0, k1)
    }
}

impl Rand for SipRng {
    fn rand<R: Rng>(other: &mut R) -> SipRng {
        let (k0, k1) = other.gen::<(u64, u64)>();
        SipRng::new(k0, k1)
    }
}


#[cfg(test)]
mod tests {
    use rand::Rng;
    use rand::os::OsRng;
    use siprng::SipRng;


    fn gen_siprng() -> SipRng {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }


    #[test]
    fn test_split_rand_independence() {
        ::tests::test_split_rand_independence(&mut gen_siprng());
    }

    #[test]
    fn test_split_rand_closure() {
        ::tests::test_split_rand_closure(&mut gen_siprng());
    }

    #[test]
    fn test_split_rand_split() {
        ::tests::test_split_rand_split(&mut gen_siprng());
    }


    fn gen_seed() -> (u64, u64) {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }

    #[test]
    fn test_rng_rand_seeded() {
        let seed = gen_seed();
        ::tests::test_rng_rand_seeded::<SipRng, (u64, u64)>(seed);
    }

    #[test]
    fn test_rng_seeded() {
        let seed = gen_seed();
        ::tests::test_rng_seeded::<SipRng, (u64, u64)>(seed);
    }

    #[test]
    fn test_rng_reseed() {
        let seed = gen_seed();
        ::tests::test_rng_reseed::<SipRng, (u64, u64)>(seed);
    }

}
