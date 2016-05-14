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

pub mod siprng;

use rand::{Rng, Rand};
use std::hash::{Hash, Hasher, SipHasher};

/// A trait for **splittable** pseudo random generators.  
pub trait SplitRng : Rng + Sized {
    
    /// The type of branches produced off a `SplitRng` instance.  A
    /// branch is an immutable object that captures the state of the
    /// RNG at the branching point, and serves as a factory for
    /// constructing RNGs for the "children."
    type Branch : RngBranch<Self>;
    
    /// Split this generator into branches.  Each branch is accessible
    /// from the resulting `Branch` object as a unique `usize` index.
    ///
    /// The original generator is moved into this function, and can
    /// then no longer be reused.  This is deliberate; the Claessen &
    /// Pałka splittable RNG construction requires this.
    ///
    /// But note that the `Branch` object returned from here supports
    /// instantiating the same branch multiple times.  This is also
    /// deliberate.
    fn splitn(self) -> Self::Branch;
    
    /// Split this random number generator into two children.  This
    /// has a default implementation in terms of `splitn`.
    fn split(self) -> (Self, Self) {
        let branches: Self::Branch = self.splitn();
        (branches.branch(0), branches.branch(1))
    }
    
}

/// The trait implemented by the branchea of a `SplitRng`.  These
/// objects act as immutable factories for `SplitRng` instances,
/// accessed by supplying an `usize` index.
pub trait RngBranch<R> {
    /// Instantiate the `i`th branch of the captured `SplitRng`.
    ///
    /// Note that instantiating the same `i` multiple times is
    /// allowed, and they all start from the same state.  This is
    /// useful in some cases; for example, random generation of
    /// deterministic functions (like Haskell's QuickCheck library
    /// does).
    fn branch(&self, i: usize) -> R;
}


/// A type that can be randomly generated from an `RngBranch`.  
///
/// Note that any `Rand` type can be trivially made `SplitRand` but
/// not vice-versa.  The reason is that a `SplitRand` type may be one
/// whose generation needs the additional power afforded by a
/// `RngBranch` and `SplitRng`.
///
/// The "killer app" for this trait is random generation of
/// deterministic closures.  Yes, you read that right:
///
/// * Each generated closure is **deterministic**: it maps equal
///   arguments to equal results on succesive calls.
/// * The generated closures are **random**: the deterministic mapping
///   that each one implements is randomly chosen.
///
/// This works because an `RngBranch` is a deterministic factory of
/// `SplitRng`s--it's a function from branch numbers to `SplitRng`s at
/// their initial states.
///
/// **This feature is experimental, and its API may change.**
pub trait SplitRand {
    
    /// Generates a random instance of this type using the
    /// specified source of randomness.
    fn rand<R, S>(branch: &S) -> Self
        where R: Rng, S: RngBranch<R>, S: Clone;
    
}

impl<A: Hash, B: Rand> SplitRand for Box<Fn(A) -> B> {
    fn rand<R, S>(branch: &S) -> Self 
        where R: Rng, S: RngBranch<R>, S: Clone, S: 'static
    {
        fn hash<T: Hash>(t: &T) -> u64 {
            let mut s = SipHasher::new();
            t.hash(&mut s);
            s.finish()
        }
        
        let branch = branch.clone();
        Box::new(move |arg: A| {
            Rand::rand(&mut branch.branch(hash(&arg) as usize))
        })
    }        
}


