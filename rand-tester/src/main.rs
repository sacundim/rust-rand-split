//! A simple program demonstrating a variety of random number
//! generators, some from the standard `rand` library, some from
//! `rust-crypto` and those from my `rand-split`.

extern crate clap;
extern crate crypto;
extern crate mersenne_twister;
extern crate rand;
extern crate rand_split;
extern crate void;

use clap::{App, SubCommand};

use crypto::fortuna::{self, Fortuna};

use mersenne_twister::{MersenneTwister, MT19937, MT19937_64};

use rand::{Rand, Rng, SeedableRng, StdRng, XorShiftRng};
use rand::chacha::ChaChaRng;
use rand::isaac::{IsaacRng, Isaac64Rng};
use rand::os::OsRng;

use rand_split::siprng::SipRng;
use rand_split::chaskeyrng::ChaskeyRng;
use rand_split::twolcg::TwoLcgRng;

use std::io::{self, Write};

use void::{Void, unreachable};


fn main() {
    match dispatch() {
        Err(err) => exit_with_error(err),
        Ok(void) => unreachable(void),
    }
}

fn exit_with_error(error: io::Error) -> ! {
    let mut stderr = std::io::stderr();
    let _ = writeln!(&mut stderr, "Program failed with error: {}", error);
    std::process::exit(1);
}

fn dispatch() -> io::Result<Void> {
    let app = App::new("PRNG Example Program")
        .version("0.1")
        .about("Provides quick access to the raw output of several RNGs.")
        .subcommand(SubCommand::with_name("fortuna")
                    .about("The Fortuna CSPRNG, from rust-crypto"))
        .subcommand(SubCommand::with_name("chacha")
                    .about("The ChaCha PRNG, from random"))
        .subcommand(SubCommand::with_name("isaac")
                    .about("The Isaac PRNG, from random"))
        .subcommand(SubCommand::with_name("isaac64")
                    .about("The Isaac64 PRNG, from random"))
        .subcommand(SubCommand::with_name("os")
                    .about("The OsRng PRNG, from random"))
        .subcommand(SubCommand::with_name("std")
                    .about("The StdRng, from random"))
        .subcommand(SubCommand::with_name("xorshift")
                    .about("The XorShift PRNG, from random"))
        .subcommand(SubCommand::with_name("mt")
                    .about("Mersenne Twister (platform-appropriate)"))
        .subcommand(SubCommand::with_name("mt19937")
                    .about("Mersenne Twister (32-bit)"))
        .subcommand(SubCommand::with_name("mt19937_64")
                    .about("Mersenne Twister (64-bit)"))
        .subcommand(SubCommand::with_name("siprng")
                    .about("The siprng generator, from rand-split"))
        .subcommand(SubCommand::with_name("chaskey")
                    .about("The chaskey generator, from rand-split"))
        .subcommand(SubCommand::with_name("twolcg")
                    .about("The TwoLCG generator, from rand-split"))
        ;
    let matches = app.get_matches();

    Ok(match matches.subcommand_name() {
        Some("chacha") => try!(run_rng::<ChaChaRng>()),
        Some("isaac") => try!(run_rng::<IsaacRng>()),
        Some("isaac64") => try!(run_rng::<Isaac64Rng>()),
        Some("os") => try!(write_forever(&mut try!(StdRng::new()))),
        Some("std") => try!(write_forever(&mut try!(StdRng::new()))),
        Some("xorshift") => try!(run_rng::<XorShiftRng>()),
        Some("mt") => try!(run_rng::<MersenneTwister>()),
        Some("mt19937") => try!(run_rng::<MT19937>()),
        Some("mt19937_64") => try!(run_rng::<MT19937_64>()),
        Some("fortuna") => try!(run_fortuna()),
        Some("siprng") => try!(run_rng::<SipRng>()),
        Some("chaskey") => try!(run_rng::<ChaskeyRng>()),
        Some("twolcg") => try!(run_rng::<TwoLcgRng>()),
        _ => print_usage(matches.usage()),
    })
}

fn print_usage(usage: &str) -> ! {
    let mut stderr = std::io::stderr();
    let _ = writeln!(&mut stderr, "{}", usage);
    std::process::exit(1);
}

fn run_rng<T: Rng + Rand>() -> io::Result<Void> {
    let mut random: T = try!(from_osrng());
    write_forever(&mut random)
}

// `Fortuna` doesn't have a `Rand` impl.
fn run_fortuna() -> io::Result<Void> {
    let mut osrng = try!(OsRng::new());
    let mut seed : [u8; fortuna::MIN_POOL_SIZE] = [0; fortuna::MIN_POOL_SIZE];
    for byte in seed.iter_mut() {
        *byte = osrng.gen();
    }
    let mut fortuna: Fortuna = SeedableRng::from_seed(&seed[..]);
    write_forever(&mut fortuna)
}

fn from_osrng<T: Rng + Rand>() -> io::Result<T> {
    let mut osrng = try!(OsRng::new());
    Ok(osrng.gen())
}

fn write_forever<T: Rng>(random: &mut T) -> io::Result<Void> {
    let mut bytes = [0u8; 65536];
    loop {
        random.fill_bytes(&mut bytes);
        try!(io::stdout().write(&bytes));
    }
}

