#![cfg_attr(test, feature(test))]

#[cfg(test)]
extern crate test;

extern crate rand;

use std::env;
use std::fmt::{self, Debug};
use std::process;
use std::path::Path;

use rand::Rng;

pub mod modprime;

pub struct HexBigint<'a>(pub &'a [u32]);

impl<'a> Debug for HexBigint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "(empty bigint)")?;
            return Ok(());
        }

        write!(f, "0x")?;
        for (i, &part) in self.0.iter().rev().enumerate() {
            if i != 0 {
                write!(f, "_")?;
            }
            write!(f, "{:08x}", part)?;
        }

        Ok(())
    }
}

fn parse_count() -> Result<u32, String> {
    let mut args = env::args();

    let prog = args.next().unwrap_or_else(|| "modprime".into());
    let prog_name = Path::new(&prog)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("modprime");

    let arg = match args.next() {
        Some(arg) => arg,
        None => return Err(format!("Usage: {} count", prog_name)),
    };

    let count = match arg.parse::<u32>() {
        Ok(count) => count,
        Err(err) => return Err(format!("{}: invalid count argument: {}", prog, err)),
    };

    Ok(count)
}

fn main() {
    let count = match parse_count() {
        Ok(count) => count,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    };

    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let mut a;
        loop {
            a = [rng.gen(), rng.gen(), rng.gen::<u32>() & 0x01ffffff];
            if a != [0, 0, 0] && a != [0xffffffff, 0xffffffff, 0x01ffffff] {
                break;
            }
        }
        let mut b;
        loop {
            b = [rng.gen(), rng.gen(), rng.gen::<u32>() & 0x01ffffff];
            if b != [0xffffffff, 0xffffffff, 0x01ffffff] {
                break;
            }
        }
        let x: [u32; 2] = rng.gen();

        let y = modprime::mod_prime(a, b, x);
        println!(
            "{:?},{:?},{:?},{:?}",
            HexBigint(&a[..]),
            HexBigint(&b[..]),
            HexBigint(&x[..]),
            HexBigint(&[y][..])
        );
    }
}
