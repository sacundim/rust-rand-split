//! A splittable pseudorandom generator based on the [TwoLCG
//! algorithm](http://on-demand.gputechconf.com/gtc/2016/presentation/s6665-guy-steele-fast-splittable.pdf).
//!
//! ## References
//!
//! * Steele, Guy.  2016.  "Fast Splittable Pseudorandom Number
//!   Generators."  Slide set at: http://on-demand.gputechconf.com/gtc/2016/presentation/s6665-guy-steele-fast-splittable.pdf

use rand::{Rand, Rng, SeedableRng};
use super::{SplitRng, SplitPrf};
use std::num::Wrapping;


/// A splittable pseudorandom generator based on the TwoLCG algorithm.
#[derive(Clone)]
pub struct TwoLcgRng {
    // The state of the generator (mutable, two words)
    s1: Wrapping<u64>,
    s2: Wrapping<u64>,

    // The parameter of the generator (immutable)
    g1: Wrapping<u64>,
    g2: Wrapping<u64>
}


/// A PRF taken off a `TwoLcgRng`.
pub struct TwoLcgPrf{
    m: Wrapping<u64>
}


impl TwoLcgRng {
    /// Create a new `TwoLcgRng` from the given state.  Note that the
    /// least significant bit of noth `g1` and `g2` are ignored.
    pub fn new(s1: u64, s2: u64, g1: u64, g2: u64) -> TwoLcgRng {
        TwoLcgRng {
            s1: Wrapping(s1),
            s2: Wrapping(s2),
            g1: Wrapping(g1 | 1u64),
            g2: Wrapping(g2 | 1u64)
        }
    }
}

impl Rng for TwoLcgRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        const MASK: Wrapping<u64> = Wrapping(0x3Fu64);
        const C0: Wrapping<u64> = Wrapping(2685821657736338717u64);
        const C1: Wrapping<u64> = Wrapping(3202034522624059733u64);
        const C2: Wrapping<u64> = Wrapping(3935559000370003845u64);

        let mut r = (self.s1 << 32) | (self.s1 >> 32);
        r ^= self.s2;
        let t = self.s1 >> 58;
        r = (r << ((t & MASK).0 as usize))
          | (r >> (((-t) & MASK).0 as usize));
        r *= C0;
        self.s1 = self.s1 * C1 + self.g1;
        self.s2 = self.s2 * C2 + self.g2;
        (r ^ (r >> 32)).0
    }
}

impl SplitRng for TwoLcgRng {
    type Prf = TwoLcgPrf;

    fn split(&mut self) -> Self {
        TwoLcgRng::from_seed(self.gen())
    }

    fn splitn(&mut self) -> TwoLcgPrf {
        TwoLcgPrf {
            m: Wrapping(self.next_u64() | 1)
        }
    }
}

impl SplitPrf<TwoLcgRng> for TwoLcgPrf {
    fn call(&self, k: u32) -> TwoLcgRng {
        /*
         * The construction in here is the one that Steele recommends
         * in the context of how to create multiple TwoLCG generators
         * upfront for a set of threads.
         */
        const FOUR: Wrapping<u64> = Wrapping(4u64);
        let (k0, k1, k2, k3) = (
            Wrapping(k     as u64),
            Wrapping((k+1) as u64),
            Wrapping((k+2) as u64),
            Wrapping((k+3) as u64),
        );
        TwoLcgRng::new((FOUR * k0 * self.m).0, 
                       (FOUR * k2 * self.m).0,
                       (FOUR * k1 * self.m).0,
                       (FOUR * k3 * self.m).0)
    }
}

impl SeedableRng<[u64; 4]> for TwoLcgRng {
    
    fn reseed(&mut self, seed: [u64; 4]) {
        self.s1 = Wrapping(seed[0]);
        self.s2 = Wrapping(seed[1]);
        self.g1 = Wrapping(seed[2] | 1);
        self.g2 = Wrapping(seed[3] | 1);
    }
    
    fn from_seed(seed: [u64; 4]) -> TwoLcgRng {
        TwoLcgRng {
            s1: Wrapping(seed[0]),
            s2: Wrapping(seed[1]),
            g1: Wrapping(seed[2] | 1u64),
            g2: Wrapping(seed[3] | 1u64)
        }
    }
}

impl Rand for TwoLcgRng {
    fn rand<R: Rng>(other: &mut R) -> TwoLcgRng {
        TwoLcgRng::from_seed(other.gen::<[u64; 4]>())
    }
}


#[cfg(test)]
mod tests {
    use rand::Rng;
    use rand::os::OsRng;
    use twolcg::TwoLcgRng;


    fn gen_twolcg() -> TwoLcgRng {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }


    #[test]
    fn test_split_rand_independence() {
        ::tests::test_split_rand_independence(&mut gen_twolcg());
    }

    #[test]
    fn test_split_rand_closure() {
        ::tests::test_split_rand_closure(&mut gen_twolcg());
    }

    #[test]
    fn test_split_rand_split() {
        ::tests::test_split_rand_split(&mut gen_twolcg());
    }


    fn gen_seed() -> [u64; 4] {
        let mut osrng = OsRng::new().ok().expect("Could not create OsRng");
        osrng.gen()
    }

    #[test]
    fn test_rng_rand_seeded() {
        let seed = gen_seed();
        ::tests::test_rng_rand_seeded::<TwoLcgRng, [u64; 4]>(seed);
    }

    #[test]
    fn test_rng_seeded() {
        let seed = gen_seed();
        ::tests::test_rng_seeded::<TwoLcgRng, [u64; 4]>(seed);
    }

    #[test]
    fn test_rng_reseed() {
        let seed = gen_seed();
        ::tests::test_rng_reseed::<TwoLcgRng, [u64; 4]>(seed);
    }
}
