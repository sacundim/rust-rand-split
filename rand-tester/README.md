# Random number generator command-line tool

This is a very simple command-line program that just spits raw output
from various Rust random number generators.  Crates covered:

* [`rand`](https://doc.rust-lang.org/rand/rand/index.html)
* [`rust-mersenne-twister`](https://dcrewi.github.io/rust-mersenne-twister/doc/0.3/mersenne_twister/index.html)
* [`rust-crypto`](https://github.com/DaGenix/rust-crypto)
* [`rand-split`](https://github.com/sacundim/rust-rand-split)

To build this tool just run from this directory:

```
cargo build --release
```

The executable program will be deposited in
`target/release/rand-tester`.  So to see what random number generators
are available, run this:

```
./target/release/rand-tester help
```

**WARNING:** You don't want to send the raw output of this program to
your terminal!  It gets... messy.  Instead, feed it to one of these
programs that tests the statistical quality of the random bytes
generated.

## PractRand

Get it from here:

* http://pracrand.sourceforge.net/

Run it this way (using Rust's `StdRng` generator):

```
./target/release/rand-tester std |/path/to/PractRand/RNG_test stdin
```

## Dieharder

Get it from here:

* https://www.phy.duke.edu/~rgb/General/dieharder.php

Run it this way:

```
./target/release/rand-tester fortuna |/path/to/dieharder -a -g 200
```
