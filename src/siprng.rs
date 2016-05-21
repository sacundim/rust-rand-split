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
//! An outline of how this generator works, conceptually:
//!
//! 1. We SipHash as a **pseudo-random function** (PRF), keyed with
//!    the RNG seed (128 bits).
//! 2. The generator (conceptually) records the history of operations
//!    that have been invoked on it and its "parents."
//! 3. To generate random output, this history is interpreted as a 
//!    string and hashed.
//!
//! By computing the PRF incrementally this can be done in constant
//! time and space.  I.e., the generator's state is the intermediate
//! state of hashing the prefix of the operation string that it has
//! seen so far.
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
use std::u32;


/// A splittable pseudorandom generator based on SipHash.
pub struct SipRng {
    v0:  u64,
    v1:  u64,
    v2:  u64,
    v3:  u64,
    ctr: u32,
    len: u8
}

/// A PRF taken off a `SipRng`.
pub struct SipPrf(SipRng);


/// A round of the SipHash function.
macro_rules! sip_round {
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

/// Process one block of SipHash.  One block = one `u64`.
macro_rules! sip_block {
    ($v0: expr, $v1: expr, $v2: expr, $v3: expr, $block: expr) => {
        $v3 ^= $block;
        sip_round!($v0, $v1, $v2, $v3);
        sip_round!($v0, $v1, $v2, $v3);
        $v0 ^= $block;
    }
}

/// Compute the result of SipHash.  `$len` is the amount of data
/// hashed, in bytes.
macro_rules! sip_finish {
    ($v0: expr, $v1: expr, $v2: expr, $v3: expr, $len: expr) => {
        {
            sip_block!($v0, $v1, $v2, $v3, 
                       ($len as u64).wrapping_shl(59));
            
            $v2 ^= 0xff;
            sip_round!($v0, $v1, $v2, $v3);
            sip_round!($v0, $v1, $v2, $v3);
            sip_round!($v0, $v1, $v2, $v3);
            sip_round!($v0, $v1, $v2, $v3);
            $v0 ^ $v1 ^ $v2 ^ $v3
        }
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
            len: 0
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


    /*
     * The generator works by encoding execution traces as two kinds
     * of 64-bit blocks that we feed to SipHash:
     *
     * 1. A **counter block**, that records a sequence of `advance`
     *    operations;
     * 2. A **split block**, that records a single split operation
     *    and its branch number.
     *
     * A counter block is an u64 value that encodes a u32 counter in
     * its least significant bits, and all zeroes in the most
     * significant bits.  A split block is an u64 that encodes an u32
     * branch number in its LSBs, and all ones in its MSBs.
     */


    /// Generate one block of sequential output.
    #[inline]
    fn advance(&mut self) -> u64 {
        let result: u64 = {
            // Compute a hash result.  This doesn't mutate the
            // generator state.
            let (mut v0, mut v1, mut v2, mut v3) = 
                (self.v0, self.v1, self.v2, self.v3);
            sip_block!(v0, v1, v2, v3, self.ctr as u64);
            sip_finish!(v0, v1, v2, v3, (self.len + 1).wrapping_mul(8))
        };

        self.ctr = if self.ctr == u32::MAX {
            // We're about to overflow the counter.  We avoid a
            // cycle by descending into a branch.
            self.descend(0);
            0
        } else {
            self.ctr.wrapping_add(1)
        };
         
        result
    }

    /// "Descend" into a numbered branch.
    #[inline]
    fn descend(&mut self, i: u32) {
        sip_block!(self.v0, self.v1, self.v2, self.v3, self.ctr as u64);
        sip_block!(self.v0, self.v1, self.v2, self.v3, 
                   (i as u64) | 0xffffffff00000000);
        self.len = self.len.wrapping_add(2);
        self.ctr = 0;
    }

}

impl SplitPrf<SipRng> for SipPrf {
    fn call(&self, i: u32) -> SipRng {
        let mut r = self.0.clone();
        r.descend(i);
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
        self.advance()
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
        self.len = 0;
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
