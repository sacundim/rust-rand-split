//! A splittable pseudorandom generator based on the [Chaskey
//! MAC](http://mouha.be/chaskey/).  **This is not intended to be a
//! cryptographically secure PRNG.**
//!
//! Like `SipRng`, this is broadly modeled after Claessen and Pałka's
//! splittable PRNGs, but with a different choice of cryptographic
//! primitive.
//!
//! ## References
//!
//! * Mouha, Nicky, Bart Mennik, Anthony Van Herrewege, Dai Watanabe,
//!   Bart Preneet and Ingrid Verbauwhede.  2014.  ["Chaskey: An
//!   Efficient MAC Algorithm for 32-bit
//!   Microcontrollers."](https://eprint.iacr.org/2014/386.pdf)
//!   Cryptology ePrint Archive, Report 2014/386.

//! * Claessen, Koen and Michał H. Pałka.  2013.  ["Splittable
//!   Pseudorandom Number Generators using Cryptographic
//!   Hashing."](http://publications.lib.chalmers.se/records/fulltext/183348/local_183348.pdf)
//!   *Haskell '13: Proceedings of the 2013 ACM SIGPLAN symposium on
//!   Haskell*, pp. 47-58.


use rand::{Rand, Rng, SeedableRng};
use super::{SplitRng, SplitPrf};
use std::u32;


/// A splittable pseudorandom generator based on Chaskey.
pub struct ChaskeyRng {
    // The state of the splittable RNG, properly speaking.
      v: [u32; 4],
     k1: [u32; 4],
    ctr: u64,

    // We buffer the 128-bit raw outputs of the RNG to speed it up a bit.
    buf: [u32; 4],
    i: usize
}

/// A PRF taken off a `ChaskeyRng`.
pub struct ChaskeyPrf(ChaskeyRng);


impl ChaskeyRng {
    pub fn new(seed: [u32; 4]) -> ChaskeyRng {
        let mut result = ChaskeyRng { 
              v: seed,
             k1: times_two(seed),
            ctr: 0,
            buf: [0u32; 4],
              i: 0
        };
        result.advance();
        result
    }

    fn reseed(&mut self, seed: [u32; 4]) {
        self.v = seed;
        self.k1 = times_two(seed);
        self.ctr = 0;
        self.buf = [0u32; 4];
        self.i = 0;
        self.advance();
    }
    
    fn clone(&self) -> ChaskeyRng {
        ChaskeyRng {
              v: self.v,
             k1: self.k1,
            ctr: self.ctr,
            buf: self.buf,
              i: self.i
        }
    }

    #[inline]
    fn advance(&mut self) {
        let block = [0, 0, lsb32(self.ctr), msb32(self.ctr)];
        xor_u32x4(&mut self.buf, &block);
        xor_u32x4(&mut self.buf, &self.k1);
        permute8(&mut self.buf);
        xor_u32x4(&mut self.buf, &self.k1);

        self.ctr = self.ctr.wrapping_add(1);
        self.i = 0;
    }

    #[inline]
    fn descend(&mut self, i: u32) {
        let block = [u32::MAX, i, lsb32(self.ctr), msb32(self.ctr)];
        xor_u32x4(&mut self.buf, &block);
        permute8(&mut self.v);

        self.ctr = 0;
        self.advance();
    }

}

impl SplitPrf<ChaskeyRng> for ChaskeyPrf {
    fn call(&self, i: u32) -> ChaskeyRng {
        let mut r = self.0.clone();
        r.descend(i);
        r
    }
}

impl SplitRng for ChaskeyRng {
    type Prf = ChaskeyPrf;

    fn split(&mut self) -> Self {
        let mut child = self.clone();
        self.descend(0);
        child.descend(1);
        child
    }

    fn splitn(&mut self) -> ChaskeyPrf {
        ChaskeyPrf(self.split())
    }

}

impl Rng for ChaskeyRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        if self.i >= 4 {
            self.advance();
        } 
        let result = self.buf[self.i];
        self.i += 1;
        result
    }    
}

impl SeedableRng<[u32; 4]> for ChaskeyRng {
    
    fn reseed(&mut self, seed: [u32; 4]) {
        ChaskeyRng::reseed(self, seed);
    }
    
    fn from_seed(seed: [u32; 4]) -> ChaskeyRng {
        ChaskeyRng::new(seed)
    }
}

impl Rand for ChaskeyRng {
    fn rand<R: Rng>(other: &mut R) -> ChaskeyRng {
        ChaskeyRng::new(other.gen::<[u32; 4]>())
    }
}


/*
 * Chaskey's building blocks.
 */

/// Function used in the Chaskey key schedule.
#[inline(always)]
pub fn times_two(key: [u32; 4]) -> [u32; 4] {
    const C: [u32; 2] = [0x00, 0x87];
    [key[0].wrapping_shl(1) ^ C[key[3].wrapping_shr(31) as usize],
     key[1].wrapping_shl(1) ^ key[0].wrapping_shr(31),
     key[2].wrapping_shl(1) ^ key[1].wrapping_shr(31),
     key[3].wrapping_shl(1) ^ key[2].wrapping_shr(31)]
}

/// XOR a `[u32; 4]` value into the Chaskey state.
#[inline(always)]
pub fn xor_u32x4(state: &mut [u32; 4], block: &[u32; 4]) {
    state[0] ^= block[0];
    state[1] ^= block[1];
    state[2] ^= block[2];
    state[3] ^= block[3];
}

// The original Chaskey permutation (8 rounds).
#[inline(always)]
pub fn permute8(state: &mut [u32; 4]) {
    permute4(state); permute4(state);
}

#[inline(always)]
fn permute4(state: &mut [u32; 4]) {
    round(state); round(state); 
    round(state); round(state);
}

/// The Chaskey round function.
#[inline(always)]
pub fn round(v: &mut [u32; 4]) {
    v[0]  = v[0].wrapping_add(v[1]); v[2]  = v[2].wrapping_add(v[3]);
    v[1]  = v[1].rotate_left(5);     v[3]  = v[3].rotate_left(8);
    v[1] ^= v[0];                    v[3] ^= v[2];
    v[0]  = v[0].rotate_left(16);
    
    v[2]  = v[2].wrapping_add(v[1]); v[0]  = v[0].wrapping_add(v[3]);
    v[1]  = v[1].rotate_left(7);     v[3]  = v[3].rotate_left(13);
    v[1] ^= v[2];                    v[3] ^= v[0];
    v[2]  = v[2].rotate_left(16);    
}

/*
 * Utility functions.
 */

/// Take the least 32 significant bits of an `u64`.
#[inline(always)]
fn lsb32(n: u64) -> u32 {
    const MASK: u64 = 0xffff_ffff;
    (n & MASK) as u32
}

/// Take the most 32 significant bits of an `u64`.
#[inline(always)]
fn msb32(n: u64) -> u32 {
    lsb32(n.wrapping_shr(32))
}


#[cfg(test)]
mod tests {
    use rand::Rng;
    use rand::os::OsRng;
    use chaskeyrng::ChaskeyRng;


    fn gen_chaskeyrng() -> ChaskeyRng {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }


    #[test]
    fn test_split_rand_independence() {
        ::tests::test_split_rand_independence(&mut gen_chaskeyrng());
    }

    #[test]
    fn test_split_rand_closure() {
        ::tests::test_split_rand_closure(&mut gen_chaskeyrng());
    }

    #[test]
    fn test_split_rand_split() {
        ::tests::test_split_rand_split(&mut gen_chaskeyrng());
    }


    fn gen_seed() -> [u32; 4] {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }

    #[test]
    fn test_rng_rand_seeded() {
        let seed = gen_seed();
        ::tests::test_rng_rand_seeded::<ChaskeyRng, [u32; 4]>(seed);
    }

    #[test]
    fn test_rng_seeded() {
        let seed = gen_seed();
        ::tests::test_rng_seeded::<ChaskeyRng, [u32; 4]>(seed);
    }

    #[test]
    fn test_rng_reseed() {
        let seed = gen_seed();
        ::tests::test_rng_reseed::<ChaskeyRng, [u32; 4]>(seed);
    }
}
