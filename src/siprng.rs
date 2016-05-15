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
//! References:
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
#[derive(Clone)]
pub struct SipRng {
    v0:  u64,
    v1:  u64,
    v2:  u64,
    v3:  u64,
    ctr: u64,
    len: usize
}

/// A PRF taken off a `SipRng`.
#[derive(Clone)]
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

    #[inline]
    fn advance(&mut self) {
        self.v3 ^= self.ctr;
        sipround!(self.v0, self.v1, self.v2, self.v3);
        self.v0 ^= self.ctr;
        self.ctr = self.ctr.wrapping_add(1);
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

    fn splitn(self) -> SipPrf {
        SipPrf(self)
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
    /*
     * I have shamelessly lifted a lot of these tests from the `rand`
     * crate.
     */

    use rand::{Rng, SeedableRng, Rand};
    use rand::os::OsRng;
    use siprng::SipRng;
    use ::{SplitRng, SplitRand};


    #[test]
    fn test_rng_rand_split() {
        let seed : (u64, u64) = gen_seed();

        let mut ra: SipRng = SeedableRng::from_seed(seed);
        let mut rb: SipRng = SeedableRng::from_seed(seed);

        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
        
        let (mut ra0, mut ra1) = ra.split();
        let (mut rb0, mut rb1) = rb.split();

        assert!(iter_eq(ra0.gen_ascii_chars().take(100),
                        rb0.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra1.gen_ascii_chars().take(100),
                        rb1.gen_ascii_chars().take(100)));

        let (mut ra00, mut ra01) = ra0.split();
        let (mut ra10, mut ra11) = ra1.split();
        let (mut rb00, mut rb01) = rb0.split();
        let (mut rb10, mut rb11) = rb1.split();

        assert!(iter_eq(ra00.gen_ascii_chars().take(100),
                        rb00.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra01.gen_ascii_chars().take(100),
                        rb01.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra10.gen_ascii_chars().take(100),
                        rb10.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra11.gen_ascii_chars().take(100),
                        rb11.gen_ascii_chars().take(100)));
    }

    #[test]
    fn test_rng_rand_closure() {
        type F = Box<Fn([u64; 8]) -> [u64; 8]>;

        let seed : (u64, u64) = gen_seed();

        let ra: SipRng = SeedableRng::from_seed(seed);
        let rb: SipRng = SeedableRng::from_seed(seed);
        let fa: F = SplitRand::rand(ra);
        let fb: F = SplitRand::rand(rb);

        let mut rc: SipRng = SeedableRng::from_seed(seed);
        for _ in 0..100 {
            let x: [u64; 8] = Rand::rand(&mut rc);
            let ya = fa(x);
            let yb = fb(x);
            assert_eq!(ya, yb);
        }
    }

    #[test]
    fn test_rng_rand_seeded() {
        let seed : (u64, u64) = gen_seed();

        let mut ra: SipRng = SeedableRng::from_seed(seed);
        let mut rb: SipRng = SeedableRng::from_seed(seed);

        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
    }

    #[test]
    fn test_rng_seeded() {
        let seed : (u64, u64) = gen_seed();
        let mut ra: SipRng = SeedableRng::from_seed(seed);
        let mut rb: SipRng = SeedableRng::from_seed(seed);
        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
    }

    #[test]
    fn test_rng_reseed() {
        let seed : (u64, u64) = (1234567890, 987654321);
        let mut r: SipRng = SeedableRng::from_seed(seed);
        let string1: String = r.gen_ascii_chars().take(100).collect();

        r.reseed(seed);

        let string2: String = r.gen_ascii_chars().take(100).collect();
        assert_eq!(string1, string2);
    }

    #[test]
    fn test_rng_clone() {
        let seed : (u64, u64) = (0, 0);
        let mut rng: SipRng = SeedableRng::from_seed(seed);
        let mut clone = rng.clone();
        for _ in 0..16 {
            assert_eq!(rng.next_u64(), clone.next_u64());
        }
    }

    /*
     * Utility functions
     */

    fn gen_seed() -> (u64, u64) {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }

    fn iter_eq<I, J>(i: I, j: J) -> bool
        where I: IntoIterator,
              J: IntoIterator<Item=I::Item>,
              I::Item: Eq
    {
        // make sure the iterators have equal length
        let mut i = i.into_iter();
        let mut j = j.into_iter();
        loop {
            match (i.next(), j.next()) {
                (Some(ref ei), Some(ref ej)) if ei == ej => { }
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}
