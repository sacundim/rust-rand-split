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
use std::mem;
use std::u32;


/// A splittable pseudorandom generator based on Chaskey.
pub struct ChaskeyRng {
      v: [u32; 4],
     k1: [u32; 4],
    ctr: u64
}

/// A PRF taken off a `ChaskeyRng`.
pub struct ChaskeyPrf(ChaskeyRng);


macro_rules! round {
    ($v0: expr, $v1: expr, $v2: expr, $v3: expr) => {
        $v0 = $v0.wrapping_add($v1); $v2 = $v2.wrapping_add($v3);
        $v1 = $v1.rotate_left(5);    $v3 = $v3.rotate_left(8);
        $v1 ^= $v0;                  $v3 ^= $v2;
        $v0 = $v0.rotate_left(16);

        $v2 = $v2.wrapping_add($v1); $v0 = $v0.wrapping_add($v3);
        $v1 = $v1.rotate_left(7);    $v3 = $v3.rotate_left(13);
        $v1 ^= $v2;                  $v3 ^= $v0;
        $v2 = $v2.rotate_left(16);
    }
}

macro_rules! permute {
    ($v0: expr, $v1: expr, $v2: expr, $v3: expr) => {
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
        round!($v0, $v1, $v2, $v3);
    }
}

macro_rules! times_two {
    ($k: expr) => {
        [$k[0].wrapping_shl(1) 
         ^ if $k[3] < 0x8000_0000 { 0 } else { 0x87 },
         $k[1].wrapping_shl(1) ^ 0,
         $k[2].wrapping_shl(1) ^ 0,
         $k[3].wrapping_shl(1) ^ 0]
    }
}

#[inline]
fn lsb32(n: u64) -> u32 {
    const MASK: u64 = 0xffff_ffff;
    (n & MASK) as u32
}

#[inline]
fn msb32(n: u64) -> u32 {
    lsb32(n.wrapping_shr(32))
}

impl ChaskeyRng {
    pub fn new(k: [u32; 4]) -> ChaskeyRng {
        ChaskeyRng { 
              v: k,
             k1: times_two!(k),
            ctr: 0
        }
    }

    fn clone(&self) -> ChaskeyRng {
        ChaskeyRng {
              v: self.v,
             k1: self.k1,
            ctr: self.ctr
        }
    }

    #[inline]
    fn advance(&mut self) -> [u32; 4] {
        let result = {
            let (mut v0, mut v1, mut v2, mut v3) = 
                (self.v[0] ^ self.k1[0], 
                 self.v[1] ^ self.k1[1],
                 self.v[2] ^ self.k1[2] ^ lsb32(self.ctr), 
                 self.v[3] ^ self.k1[3] ^ msb32(self.ctr));
            permute!(v0, v1, v2, v3);
            [v0 ^ self.k1[0], 
             v1 ^ self.k1[1],
             v2 ^ self.k1[2], 
             v3 ^ self.k1[3]]
        };
        self.ctr = self.ctr.wrapping_add(1);

        result
    }

    #[inline]
    fn descend(&mut self, i: u32) {
        self.v = [self.v[0] ^ u32::MAX, 
                  self.v[1] ^ i,
                  self.v[2] ^ lsb32(self.ctr), 
                  self.v[3] ^ msb32(self.ctr)];
        permute!(self.v[0], self.v[1], self.v[2], self.v[3]);
        self.ctr = 0;
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
        let child = self.split();
        ChaskeyPrf(child)
    }

}

impl Rng for ChaskeyRng {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        let block = self.advance();
        (block[0] as u64).wrapping_shl(32) | (block[1] as u64)
    }
    
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.advance()[0]
    }
    
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(16) {
            let block = unsafe {
                mem::transmute::<[u32; 4], [u8; 16]>(self.advance())
            };
            for i in 0..chunk.len() {
                chunk[i] = block[i];
            }
        }
    }
}

impl SeedableRng<[u32; 4]> for ChaskeyRng {
    
    fn reseed(&mut self, seed: [u32; 4]) {
        self.v = seed;
        self.k1 = times_two!(seed);
        self.ctr = 0;
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


#[cfg(test)]
mod tests {
    use rand::Rng;
    use rand::os::OsRng;
    use chaskeyrng::ChaskeyRng;


    fn gen_siprng() -> ChaskeyRng {
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
