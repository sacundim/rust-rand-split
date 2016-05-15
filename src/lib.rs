// Copyright 2016 Luis Casillas. See the COPYRIGHT file at the
// top-level directory of this distribution
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A crate to support **splittable** random number generators.
//!
//! These are genertors that support a "split" operation that produces
//! two or more "child" generators whose initial state depends on the
//! source RNG's state, but produce outputs that are uncorrelated to
//! each other.  This can be useful in some contexts:
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
//! References:
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
    fn call(&self, i: u64) -> Rng;
}

/// A type that can be randomly generated from a `SplitRand`.
/// Implementations are expected to exploit splittability where
/// possible.
pub trait SplitRand {
    
    /// Generates a random instance of this type using the given
    /// source of randomness.
    fn split_rand<R: SplitRng>(rng: &mut R) -> Self;
    
}

impl<A: Hash, B: Rand> SplitRand for Box<Fn(A) -> B> {
    
    fn split_rand<R>(rng: &mut R) -> Self 
        where R: SplitRng, R: 'static
    {
        fn hash<T: Hash>(t: &T) -> u64 {
            let mut s = SipHasher::new();
            t.hash(&mut s);
            s.finish()
        }
        
        let prf = rng.splitn();
        Box::new(move |arg: A| {
            Rand::rand(&mut prf.call(hash(&arg)))
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
                Rand::rand(rng)
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
