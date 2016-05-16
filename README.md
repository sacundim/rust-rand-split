# Splittable Pseudorandom Number Generation for Rust

[![Build Status](https://travis-ci.org/sacundim/rust-rand-split.svg?branch=master)](https://travis-ci.org/sacundim/rust-rand-split)

This crate provides traits for **splittable** pseudorandom number
generators, and a simple implementation based on SipHash.


## This is not a cryptographic random number generator

While the random number generator included here (`SipRng`) is based on
a cryptographic primitive, **it has not been designed or evaluated for
security**.


## Documentation

[**Documentation**](http://sacundim.github.io/rust-rand-split/)


## TODO/nice-to-haves

* Integration with some sort of lazy evaluation mechanism.
* Integration with some sort of parallel execution mechanism.


## References

* Aumasson, Jean-Philippe and Daniel J. Bernstein.  2012.
  ["SipHash: a fast short-input PRF."](https://eprint.iacr.org/2012/351)
  Cryptology ePrint Archive, Report 2012/351.
* Claessen, Koen and Michał H. Pałka.  2013.  ["Splittable
  Pseudorandom Number Generators using Cryptographic
  Hashing."](http://publications.lib.chalmers.se/records/fulltext/183348/local_183348.pdf)
  *Haskell '13: Proceedings of the 2013 ACM SIGPLAN symposium on
  Haskell*, pp. 47-58.
* Pałka, Michał H.
  [`tf-random` Haskell library](https://hackage.haskell.org/package/tf-random).
* Schaathun, Hans Georg.  2015.
  ["Evaluation of Splittable Pseudo-Random Generators."](http://www.hg.schaathun.net/research/Papers/hgs2015jfp.pdf)
  *Journal of Functional Programming*, Vol. 25.
