// Copyright 2016 Luis Casillas. See the COPYRIGHT file at the
// top-level directory of this distribution
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Splittable pseudo-random number generators.
//!
//! These are generators that support a **split** operations that
//! forks one or more "child" generators off the parent.  Children's
//! initial states depend on the source RNG's state, but they produce
//! outputs that are uncorrelated to each other.  This can be useful
//! in some contexts:
//!
//! * Functional programs sometimes make good use of splittable RNGs.  The
//!   Haskell [QuickCheck](https://hackage.haskell.org/package/QuickCheck) 
//!   library is a notable example that uses a splittable, pure-functional
//!   PRNG to support random generation of deterministic functions.
//! * Parallel programs may benefit from a splittable PRNG.  The
//!   advantage isn't necessarily performance but rather
//!   **reproducibility**; it makes it easier to guarantee that using
//!   the same seed yields the same results even if the execution order
//!   is nondeterministic.
//!
//! 
//! ## Motivation
//! 
//! Splittable PRNGs offer superior **reproducibility** in some
//! scenarios.  Consider a case where you are randomly generating a
//! pair of `Rand` instances with a conventional, i.e. **sequential**
//! PRNG:
//! 
//! ```should_panic
//! extern crate rand;
//! use rand::{Rng, SeedableRng, XorShiftRng, Rand, thread_rng};
//!
//! # fn main() {
//! // Make two PRNGs from the same seed.
//! let seed: [u32; 4] = thread_rng().gen();
//! let mut rng0: XorShiftRng = SeedableRng::from_seed(seed);
//! let mut rng1: XorShiftRng = SeedableRng::from_seed(seed);
//! 
//! type T0 = [u64; 16];
//! type T1 = [u64; 32];
//! let (_, a): (T0, T0) = Rand::rand(&mut rng0);
//! let (_, b): (T1, T0) = Rand::rand(&mut rng1);
//!
//! // This should fail, unless you're crazy unlucky:
//! assert_eq!(a, b); 
//! # }
//! ```
//! 
//! With a sequential generator, the way this is generated typically
//! is so:
//! 
//! 1. The `rng` starts at some initial state.  Call this state `s0`.
//! 2. We use `rng` to generate a random `a`.  This leaves the `rng`
//!    in some state, call it `s1`.
//! 3. We reuse `rng` to generate a random `b`.  This leaves the `rng`
//!    in some state, call it `s2`.
//! 
//! Now, if the logic that generates the `a` value changes, then
//! states `s1` may end up different even if we start from the same
//! `s0`, which means that the value generated for `b` may also
//! change.
//! 
//! With a splittable PRNG, you can generate the tuple's elements with
//! **independent** PRNGs split off the parent, so that changes to the
//! logic that generates one element cannot affect the other.  This
//! can be illustrated with our flagship unit test:
//! 
//! ```
//! extern crate rand;
//! extern crate rand_split;
//!
//! use rand::{Rng, Isaac64Rng, thread_rng};
//! use rand_split::{SplitRng, SplitPrf, SplitRand, Split, Prf};
//!
//! # fn main() {
//! // We will be using the `rand` crate's `Isaac64Rng`, but
//! // wrapped with a `Split` wrapper that adds splittability
//! // on top of it.
//! type OurRng = Split<Isaac64Rng>;
//! type OurPrf = Prf<Isaac64Rng>;
//!
//! // The library's RNGs have `Rand` instances, so we can get
//! // a randomly seeded RNG this way:
//! let mut rng: OurRng = thread_rng().gen();
//!
//! // We split off a **pseudo-random function** ("PRF") from
//! // the RNG.  PRFs implement the `SplitPrf` trait.
//! let prf: OurPrf = rng.splitn();
//! 
//! // PRFs serve as factories that construct further `SplitRng`s.
//! // So now we pick a random index and call the PRF four times
//! // with that index to get four new RNGs.
//! let i: u32 = rng.gen();
//! let mut ra: OurRng = prf.call(i);
//! let mut rb: OurRng = prf.call(i);
//! let mut rc: OurRng = prf.call(i);
//! let mut rd: OurRng = prf.call(i);
//! 
//! // A PRF is a function that captures a "frozen" state from
//! // its parent RNG, and constructs further RNG instances 
//! // whose initial states depend only on that frozen state and 
//! // the index supplied to `call`.  So the four RNGs we just
//! // constructed are guaranteed to be in the same initial state.
//! // And behold, they produce identical outputs!
//! assert!(iter_eq(ra.gen_ascii_chars().take(100),
//!                 rb.gen_ascii_chars().take(100)));
//! assert!(iter_eq(rc.gen_ascii_chars().take(100),
//!                 rd.gen_ascii_chars().take(100)));
//! assert!(iter_eq(ra.gen_ascii_chars().take(100),
//!                 rc.gen_ascii_chars().take(100)));
//! assert!(iter_eq(rb.gen_ascii_chars().take(100),
//!                 rd.gen_ascii_chars().take(100)));
//! 
//! // Now for the main course.  We pick two distinct types that
//! // implement the `SplitRand` trait.  We choose them so that
//! // generating a value of either type advances the state of a
//! // sequential RNG by a different amount than the other.
//! type T0 = [u64; 16];
//! type T1 = [u64; 32];
//! 
//! for _ in 0..100 {
//!     // Then we use our four initially-identical RNGs to
//!     // generate tuples representing all four combinations of
//!     // our two element types:
//!     let (a0, a1): (T0, T0) = SplitRand::split_rand(&mut ra);
//!     let (b0, b1): (T0, T1) = SplitRand::split_rand(&mut rb);
//!     let (c0, c1): (T1, T0) = SplitRand::split_rand(&mut rc);
//!     let (d0, d1): (T1, T1) = SplitRand::split_rand(&mut rd);
//!     
//!     // And here we test that the value of each element
//!     // generated depends on its type and its position within
//!     // its pair, but not on what was generated for the other
//!     // element.
//!     assert_eq!(a0, b0);
//!     assert_eq!(a1, c1);
//!     assert_eq!(b1, d1);
//!     assert_eq!(c0, d0);
//!     assert!(a0 != a1);
//!     assert!(a0 != c1);
//!     assert!(d0 != d1);
//! 
//!     // Finally, note that we're doing this inside of a loop
//!     // and reusing the same four RNGs on each iteration.  So 
//!     // at this point all four generators must end the same
//!     // state or subsequent iterations will fail.
//! }
//! # }
//! #
//! # fn iter_eq<I, J>(i: I, j: J) -> bool
//! #     where I: IntoIterator,
//! #           J: IntoIterator<Item=I::Item>,
//! #           I::Item: Eq
//! # {
//! #     // make sure the iterators have equal length
//! #     let mut i = i.into_iter();
//! #     let mut j = j.into_iter();
//! #     loop {
//! #         match (i.next(), j.next()) {
//! #             (Some(ref ei), Some(ref ej)) if ei == ej => { }
//! #             (None, None) => return true,
//! #             _ => return false,
//! #         }
//! #     }
//! # }
//! ```
//! 
//! This can be useful in programs that generate complex pseudo-random
//! data from fixed seeds, because it means that changes in one
//! location of the program can be made much less likely to affect the
//! results produced in others.  It can also be useful in parallel
//! programs, because it can be used to provide deterministic
//! pseudo-random generation in spite of non-deterministic execution
//! order.
//! 
//!
//! ## References
//!
//! * Claessen, Koen and Michał H. Pałka.  2013.  ["Splittable
//!   Pseudorandom Number Generators using Cryptographic
//!   Hashing."](http://publications.lib.chalmers.se/records/fulltext/183348/local_183348.pdf)
//!   *Haskell '13: Proceedings of the 2013 ACM SIGPLAN symposium on
//!   Haskell*, pp. 47-58.
//! * Schaathun, Hans Georg.  2015.  ["Evaluation of Splittable
//!   Pseudo-Random
//!   Generators."](http://www.hg.schaathun.net/research/Papers/hgs2015jfp.pdf)
//!   *Journal of Functional Programming*, Vol. 25.
//! * The Haskell [`tf-random` library](https://hackage.haskell.org/package/tf-random).


extern crate rand;

pub mod generic;
pub mod siprng;
pub mod chaskeyrng;

use rand::{Rng, Rand};
use siprng::{SipRng, SipPrf};
use std::hash::{Hash, Hasher, SipHasher};


/// A wrapper that generically adds splittability to RNGs.
pub type Split<Rng> = generic::Split<SipRng, Rng>;

/// The pseudo-random functions of a generic `Split` RNG.
pub type Prf<Rng> = generic::Prf<SipPrf, Rng>;


/// A trait for **splittable** pseudo random generators.  
pub trait SplitRng : Rng + Sized {
    
    /// The type of pseudo-random functions ("PRFs") produced off a
    /// `SplitRng` instance.  A PRF is a factory of `SplitRng`s, whose
    /// initial states are determined by two values:
    ///
    /// 1. The state of the `SplitRng` when the PRF was created;
    /// 2. The argument to the PRF.
    ///
    /// Note that while the term *pseudo-random function* has a
    /// technical meaning in cryptograpy, **no security claim is
    /// implied here**.
    type Prf : SplitPrf<Self>;
    
    /// Split a pseudo-random function off this generator.
    fn splitn(&mut self) -> Self::Prf;
    
    /// Split a second RNG off this one.
    fn split(&mut self) -> Self;
    
    fn split_gen<A: SplitRand>(&mut self) -> A {
        SplitRand::split_rand::<Self>(self)
    }
}

/// Pseudo-random functions ("PRFs") generated off a `SplitRng`.
///
/// A PRF is a factory of `SplitRng`s, whose initial states are
/// determined by two values:
///
/// 1. The state of the `SplitRng` when the PRF was created;
/// 2. The argument to the PRF.
///
/// Note that while the term *pseudo-random function* has a
/// technical meaning in cryptograpy, **no security claim is
/// implied here**.
pub trait SplitPrf<Rng> {
    fn call(&self, i: u32) -> Rng;
}

/// A type that can be randomly generated from a `SplitRand`.
/// Implementations are expected to exploit splittability where
/// possible.
pub trait SplitRand {
    
    /// Generates a random instance of this type using the given
    /// source of randomness.
    fn split_rand<R: SplitRng>(rng: &mut R) -> Self;
    
}

/// A newtype wrapper to add a `SplitRand` implementation to `Rand`
/// types.  This just does the same thing as the base type's `Rand`
/// one does.
pub struct Seq<A>(pub A);

impl<A: Rand> SplitRand for Seq<A> {

    #[inline]
    fn split_rand<R: SplitRng>(rng: &mut R) -> Self {
        Seq(Rand::rand(rng))
    }

}


impl<A: Hash, B: Rand> SplitRand for Box<Fn(A) -> B> {
    
    fn split_rand<R>(rng: &mut R) -> Self 
        where R: SplitRng, R: 'static
    {
        let (k0, k1) = (rng.next_u64(), rng.next_u64());
        let prf = rng.splitn();
        Box::new(move |arg: A| {
            let i: u32 = {
                // TODO: is there a way not to hardcode `SipHasher` here?
                let mut hasher = SipHasher::new_with_keys(k0, k1);
                arg.hash(&mut hasher);
                (hasher.finish() & 0xffff_ffff) as u32
            };
            Rand::rand(&mut prf.call(i))
        })
    }

}

/// A macro that implements `SplitRand` sequentially for any type that
/// has a `Rand` implementation, simply by using that.  This is meant
/// to be used for "atomic" types whose generation doesn't benefit
/// from splittability.
#[macro_export]
macro_rules! split_rand_seq_impl {
    ($t:ident) => {
        impl SplitRand for $t {
            #[inline]
            fn split_rand<R: SplitRng>(rng: &mut R) -> Self {
                let Seq(result) = SplitRand::split_rand(rng);
                result
            }
        }
    }
}

split_rand_seq_impl!{isize}
split_rand_seq_impl!{i8}
split_rand_seq_impl!{i16}
split_rand_seq_impl!{i32}
split_rand_seq_impl!{i64}

split_rand_seq_impl!{usize}
split_rand_seq_impl!{u8}
split_rand_seq_impl!{u16}
split_rand_seq_impl!{u32}
split_rand_seq_impl!{u64}

split_rand_seq_impl!{f32}
split_rand_seq_impl!{f64}
// TODO: Open01, Closed01

split_rand_seq_impl!{char}
split_rand_seq_impl!{bool}

/*
 * These macros are more or less adapted from the `rand` crate.
 */

macro_rules! tuple_impl {
    // use variables to indicate the arity of the tuple
    ($($tyvar:ident),* ) => {
        // the trailing commas are for the 1 tuple
        impl<
            $( $tyvar : SplitRand ),*
            > SplitRand for ( $( $tyvar ),* , ) {

            #[inline]
            fn split_rand<R: SplitRng>(_rng: &mut R) -> ( $( $tyvar ),* , ) {
                (
                    // use the $tyvar's to get the appropriate number of
                    // repeats (they're not actually needed)
                    $(
                        _rng.split().split_gen::<$tyvar>()
                    ),*
                    ,
                )
            }
        }
    }
}

impl SplitRand for () {
    fn split_rand<R: SplitRng>(_: &mut R) -> () { () }
}

tuple_impl!{A}
tuple_impl!{A, B}
tuple_impl!{A, B, C}
tuple_impl!{A, B, C, D}
tuple_impl!{A, B, C, D, E}
tuple_impl!{A, B, C, D, E, F}
tuple_impl!{A, B, C, D, E, F, G}
tuple_impl!{A, B, C, D, E, F, G, H}
tuple_impl!{A, B, C, D, E, F, G, H, I}
tuple_impl!{A, B, C, D, E, F, G, H, I, J}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K, L}


// Adapted, with minor modifications, from the `rand` crate.
macro_rules! array_impl {
    {$n:expr, $t:ident, $($ts:ident,)*} => {
        array_impl!{($n - 1), $($ts,)*}

        impl<T> SplitRand for [T; $n] where T: SplitRand {
            #[inline]
            fn split_rand<R: SplitRng>(_rng: &mut R) -> [T; $n] {
                [
                    _rng.split().split_gen::<$t>(), 
                    $(_rng.split().split_gen::<$ts>()),*
                ]
            }
        }
    };
    {$n:expr,} => {
        impl<T> SplitRand for [T; $n] {
            fn split_rand<R: SplitRng>(_rng: &mut R) -> [T; $n] { [] }
        }
    };
}

array_impl!{
    32, 
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, 
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T,
}


#[cfg(test)]
mod tests {
    //! These tests are reusable functions meant to be called from
    //! children modules.

    use rand::SeedableRng;
    use ::{SplitRng, SplitPrf, SplitRand};

    /// Test that generation of tuple elements with `SplitRand` is
    /// independent.
    pub fn test_split_rand_independence<R: SplitRng>(rng: &mut R) {
        let prf: R::Prf = rng.splitn();

        let i: u32 = rng.gen();
        let mut ra: R = prf.call(i);
        let mut rb: R = prf.call(i);
        let mut rc: R = prf.call(i);
        let mut rd: R = prf.call(i);

        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
        assert!(iter_eq(rc.gen_ascii_chars().take(100),
                        rd.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rc.gen_ascii_chars().take(100)));
        assert!(iter_eq(rb.gen_ascii_chars().take(100),
                        rd.gen_ascii_chars().take(100)));


        type T0 = [u64; 16];
        type T1 = [u64; 32];
        for _ in 0..100 {
            let (a0, a1): (T0, T0) = SplitRand::split_rand(&mut ra);
            let (b0, b1): (T0, T1) = SplitRand::split_rand(&mut rb);
            let (c0, c1): (T1, T0) = SplitRand::split_rand(&mut rc);
            let (d0, d1): (T1, T1) = SplitRand::split_rand(&mut rd);
            
            assert_eq!(a0, b0);
            assert_eq!(a1, c1);
            assert_eq!(b1, d1);
            assert_eq!(c0, d0);
            assert!(a0 != a1);
            assert!(a0 != c1);
            assert!(d0 != d1);
        }
    }

    /// Test generation of closures.
    pub fn test_split_rand_closure<R: SplitRng>(rng: &mut R) {
        type F = Box<Fn([u64; 8]) -> [u64; 8]>;

        let prf = rng.splitn();
        let i = rng.gen();

        let fa: F = SplitRand::split_rand(&mut prf.call(i));
        let fb: F = SplitRand::split_rand(&mut prf.call(i));
        for _ in 0..100 {
            let x: [u64; 8] = SplitRand::split_rand(rng);
            let ya = fa(x);
            let yb = fb(x);
            assert_eq!(ya, yb);
        }
    }


    /// Test that splitting a generator produces reproducible
    /// sequential results.
    pub fn test_split_rand_split<R: SplitRng>(rng: &mut R) {
        let prf = rng.splitn();
        let i = rng.gen();
        let mut ra0 = prf.call(i);
        let mut rb0 = prf.call(i);

        assert!(iter_eq(ra0.gen_ascii_chars().take(100),
                        rb0.gen_ascii_chars().take(100)));
        
        let mut ra1 = ra0.split();
        let mut rb1 = rb0.split();

        assert!(iter_eq(ra0.gen_ascii_chars().take(100),
                        rb0.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra1.gen_ascii_chars().take(100),
                        rb1.gen_ascii_chars().take(100)));
    }


    /*
     * The tests below here are lightly adapted from the `rand` crate.
     */

    pub fn test_rng_rand_seeded<R, Seed>(seed: Seed) 
        where R: SplitRng + SeedableRng<Seed>, Seed: Copy
    {
        let mut ra = R::from_seed(seed);
        let mut rb = R::from_seed(seed);
        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
    }

    pub fn test_rng_seeded<R, Seed>(seed: Seed) 
        where R: SplitRng + SeedableRng<Seed>, Seed: Copy
    {
        let mut ra = R::from_seed(seed);
        let mut rb = R::from_seed(seed);
        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
    }

    pub fn test_rng_reseed<R, Seed>(seed: Seed) 
        where R: SplitRng + SeedableRng<Seed>, Seed: Copy
    {
        let mut r = R::from_seed(seed);
        let string1: String = r.gen_ascii_chars().take(100).collect();

        r.reseed(seed);

        let string2: String = r.gen_ascii_chars().take(100).collect();
        assert_eq!(string1, string2);
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
