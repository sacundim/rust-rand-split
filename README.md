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
**independent** PRNGs split off the parent, so that changes to the
logic that generates one element cannot affect the other.  This can be
illustrated with our flagship unit test:

    /// When generating a pair with `SplitRand`, the value generated
    /// at each position in the pair should not be affected by how
    /// much randomness was consumed by the generation of the other.
    pub fn test_split_rand_independence<R: SplitRng>(rng: &mut R) {
        // First we split off a **pseudo-random function** ("PRF")
        // from the RNG, which implements the `SplitPrf` trait.  A
        // PRF, in this context, is a factory that constructs further
        // `SplitRng`s.
        let prf: R::Prf = rng.splitn();

        // Now we pick a random index, and call the PRF four times
        // with that index.
        let i: u64 = rng.next_u64();
        let mut ra: R = prf.call(i);
        let mut rb: R = prf.call(i);
        let mut rc: R = prf.call(i);
        let mut rd: R = prf.call(i);

        // A PRF is a deterministic function from the index into fresh
        // RNG instances.  So the four RNGs we just constructed are
        // guaranteed to be in the same initial state.  (Note that this
        // can be used to randomly generate pure functions!)
        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rb.gen_ascii_chars().take(100)));
        assert!(iter_eq(rc.gen_ascii_chars().take(100),
                        rd.gen_ascii_chars().take(100)));
        assert!(iter_eq(ra.gen_ascii_chars().take(100),
                        rc.gen_ascii_chars().take(100)));
        assert!(iter_eq(rb.gen_ascii_chars().take(100),
                        rd.gen_ascii_chars().take(100)));


        // We pick two distinct types that implement the `SplitRand`
        // trait.  We choose them so that generating a value of each
        // type advances the state of a sequential RNG by a different
        // amount than the other.
        type T0 = [u64; 16];
        type T1 = [u64; 32];

        for _ in 0..100 {
            // Now we use our four initially-identical RNGs to
            // generate tuples representing all four the combinations
            // of our two element types:
            let (a0, a1): (T0, T0) = SplitRand::split_rand(&mut ra);
            let (b0, b1): (T0, T1) = SplitRand::split_rand(&mut rb);
            let (c0, c1): (T1, T0) = SplitRand::split_rand(&mut rc);
            let (d0, d1): (T1, T1) = SplitRand::split_rand(&mut rd);
            
            // And here we test that the value of each element
            // generated depends on its type and its position within
            // its pair, but not on what was generated for the other
            // element.
            assert_eq!(a0, b0);
            assert_eq!(a1, c1);
            assert_eq!(b1, d1);
            assert_eq!(c0, d0);

            // Finally, note that we're doing this inside of a loop
            // and reusing the same four generators for each
            // iteration.  So at this point all four generators must
            // be in the same state for subsequent iterations to pass.
        }
    }

This can be useful in programs that generate complex pseudo-random
data from fixed seeds, because it means that changes in one location
of the program can be made much less likely to affect the results
produced in others.  It can also be useful in parallel programs,
because it can be used to provide deterministic pseudo-random
generation in spite of non-deterministic execution order.


## This is not a cryptographic random number generator

While the random number generator included here (`SipRng`) is based on
a cryptographic primitive, **it has not been designed or evaluated for
security**.


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
