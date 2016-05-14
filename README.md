# Splittable Pseudorandom Number Generation for Rust

This crate provides traits for **splittable** pseudorandom number
generators, and a simple implementation based on SipHash.


## Motivation

Splittable PRNGs offer superior **reproducibility** in some scenarios.
Consider a case where you are randomly generating a pair of `Rand`
instances with a conventional, i.e. **sequential** PRNG:

    let mut rng = new_sequential_rng();
    let (a, b) = Rand::rand(rng);

With a sequential generator, the way this is generated typically is so:

1. The `rng` starts at some initial state.  Call this state `s0`.
2. We use `rng` to generate a random `a`.  This leaves the `rng` in
   some state, call it `s1`.
3. We reuse `rng` to generate a random `b`.  This leaves the `rng` in
   some state, call it `s2`.

Now, if the logic that generates the `a` value changes, then states
`s1` may end up different even if we start from the same `s0`, which
means that the value generated for `b` may also change.

With a splittable PRNG, you can generate the tuple's elements with
**independent** PRNGs, so that the logic for generating `a` cannot
affect that for generating `b`:

    let rng = new_splittable_rng();
    let (a, b) = SplitRand::rand(rng);

This can be useful in programs that generate complex pseudo-random
data from fixed seeds, because it means that changes in one location
of the program can be made much less likely to affect the results
produced in others.

It can also be useful in parallel programs, because it can be used to
provide deterministic random generation in spite of concurrent
execution.


## This is not a cryptographic random number generator

While the random number generator included here (`SipRn`) is based on
a cryptographic primitive, **it has not been designed for security**.


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
