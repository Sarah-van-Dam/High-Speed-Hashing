#![feature(test, sip_hash_13)]

extern crate test;

use std::env;
use std::process;
use std::time::Instant;

#[allow(deprecated)]
use std::hash::{Hasher, SipHasher};

pub mod modprime;
pub mod shift;
pub mod vec;
pub mod string;

#[derive(Clone, Copy, Debug)]
pub enum OutputMode {
    Csv,
    Pretty,
}

impl OutputMode {
    pub fn is_csv(self) -> bool {
        match self {
            OutputMode::Csv => true,
            _ => false,
        }
    }

    pub fn is_pretty(self) -> bool {
        match self {
            OutputMode::Pretty => true,
            _ => true,
        }
    }
}

pub fn do_time<F>(input: &[u32], n: u32, func: F) -> f64
where
    F: Fn(u32) -> u32,
{
    let start = Instant::now();

    let mut tmp = 0;
    for _ in 0..n {
        for &x in &input[..] {
            tmp ^= func(x);
        }
    }

    let elapsed = start.elapsed();

    // Force use.
    let _ = test::black_box(tmp);

    let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1e9);
    secs
}

pub fn time_all<F>(
    mode: OutputMode,
    func_name: &str,
    data_sets: &[(&str, &[u32])],
    num_reps: u32,
    target_reps: u32,
    func: F,
) where
    F: Fn(u32) -> u32,
{
    for &(data_set_name, data_set) in data_sets {
        let num_values = data_set.len() as u32;
        let n = (target_reps + num_values - 1) / num_values;
        let actual_reps = n * num_values;

        // Run through once to warm up the caches.
        do_time(data_set, 1, &func);

        for _ in 0..num_reps {
            let secs = do_time(data_set, n, &func);
            let ns_per_value = secs * 1e9 / (actual_reps as f64);

            match mode {
                OutputMode::Csv => {
                    println!(
                        "{},{},{},{:.5},{:.5}",
                        func_name, data_set_name, n, secs, ns_per_value,
                    );
                }
                OutputMode::Pretty => {
                    println!(
                        "{}/{}, repetitions: {}, total time (s): {:.5}, time per value (ns): {:.5}",
                        func_name, data_set_name, n, secs, ns_per_value,
                    );
                }
            }
        }
    }
}

fn main() {
    // Parse '-c' or '-p' argument.

    let mut mode = None;

    let mut args = env::args();

    let argv0 = args.next().unwrap();

    for arg in args {
        match &arg[..] {
            "-c" => mode = Some(OutputMode::Csv),
            "-p" => mode = Some(OutputMode::Pretty),
            _ => {
                eprintln!("Usage: cargo run --release -- [-cp]");
                eprintln!("{}: unexpected option {:?}", argv0, arg);
                process::exit(2);
            }
        }
    }

    let mode = match mode {
        Some(mode) => mode,
        None => {
            eprintln!("Usage: cargo run --release -- [-cp]");
            eprintln!("Options:");
            eprintln!("  -c        Output as CSV.");
            eprintln!("  -p        Output as human-readable text.");
            process::exit(2);
        }
    };

    // Prepare input.

    let input_raw = include_bytes!("../flatland.txt");

    let input8 = input_raw.iter().map(|&x| x as u32).collect::<Vec<_>>();

    let input32 = input_raw[..4 * (input_raw.len() / 4)]
        .chunks(4)
        .map(|x| {
            (x[0] as u32) | ((x[1] as u32) << 8) | ((x[2] as u32) << 16) | ((x[3] as u32) << 24)
        })
        .collect::<Vec<_>>();

    let data_sets = &[
        // Flatland.txt, 8 bits at a time.
        ("flatland-8", &input8[..]),
        // Flatland.txt, 32 bits at a time.
        ("flatland-32", &input32[..]),
    ][..];

    if mode.is_csv() {
        println!("func,dataset,n,secs,nspervalue");
    }

    {
        let a = test::black_box([0x94160842, 0x674333f7, 0x005e977a]);
        let b = test::black_box([0x55387e64, 0xbdafef1f, 0x008e956a]);
        let c = test::black_box([0x55347e64, 0xb12fef1f, 0x0085951a]);
        time_all(mode, "string-poly-64", data_sets, 10, 20_000_000, |value| {
            let x = [value as u64, value as u64];
            string::poly64(a, b, c, &x)
        })
    }

    {
        let a = test::black_box(0x94160842);
        let b = test::black_box(0x674333f7);
        let c = test::black_box(0x005e977a);
        time_all(mode, "string-poly-16", data_sets, 10, 50_000_000, |value| {
            let v0 = value as u16;
            let v1 = (value >> 16) as u16;
            let x = [v0, v1, v0, v1, v0, v1, v0, v1];
            string::poly16(a, b, c, &x)
        });
    }

    // Time Vector 64x32 (pair)
    {
        let a = test::black_box([
            0x9416084294160842, 0x674333f7674333f7, 0x005e977a005e977a, 0x005e977e005e977e,
            0x9416084394160843, 0x674333f8674333f8, 0x005e977b005e977b, 0x005e977f005e977f,
            0x8416084284160842, 0xa74333f7a74333f7, 0x305e977a305e977a, 0x305e977e305e977e,
            0x8416084384160843, 0xa74333f8a74333f8, 0x305e977b305e977b, 0x305e977f305e977f,
            0x9416184294161842, 0x674343f7674343f7, 0x005ea77a005ea77a, 0x006e977e006e977e,
            0x9416184394161843, 0x674343f8674343f8, 0x005ea77b005ea77b, 0x006e977f006e977f,
            0x8416184284161842, 0xa74343f7a74343f7, 0x305ea77a305ea77a, 0x306e977e306e977e,
            0x8416184384161843, 0xa74343f8a74343f8, 0x305ea77b305ea77b, 0x306e977f306e977f,
            0xf4160842f4160842, 0xc74333f7c74333f7, 0x505e977a505e977a, 0x105e977e105e977e,
            0xf4160843f4160843, 0xc74333f8c74333f8, 0x505e977b505e977b, 0x105e977f105e977f,
            0xf4160842f4160842, 0xc74333f7c74333f7, 0x505e977a505e977a, 0x105e977e105e977e,
            0xf4160843f4160843, 0xc74333f8c74333f8, 0x505e977b505e977b, 0x105e977f105e977f,
            0xf4161842f4161842, 0xc74343f7c74343f7, 0x505ea77a505ea77a, 0x106e977e106e977e,
            0xf4161843f4161843, 0xc74343f8c74343f8, 0x505ea77b505ea77b, 0x106e977f106e977f,
            0xf4161842f4161842, 0xc74343f7c74343f7, 0x505ea77a505ea77a, 0x106e977e106e977e,
            0xf4161843f4161843, 0xc74343f8c74343f8, 0x505ea77b505ea77b, 0x106e977f106e977f,
            0xc74343f7c7434ddd,
        ]);
        time_all(mode, "prefix-64x32-pair", data_sets, 10, 20_000_000, |value| {
            let mut x = [0; 64];
            x[63] = value;
            string::prefix64x32_pair(&a, &x)
        });
    }

    // Time Vector 64x32 (pair)
    {
        let a = test::black_box([
            0x9416084294160842, 0x674333f7674333f7, 0x005e977a005e977a, 0x005e977e005e977e,
            0x9416084394160843, 0x674333f8674333f8, 0x005e977b005e977b, 0x005e977f005e977f,
            0x8416084284160842, 0xa74333f7a74333f7, 0x305e977a305e977a, 0x305e977e305e977e,
            0x8416084384160843, 0xa74333f8a74333f8, 0x305e977b305e977b, 0x305e977f305e977f,
            0x9416184294161842, 0x674343f7674343f7, 0x005ea77a005ea77a, 0x006e977e006e977e,
            0x9416184394161843, 0x674343f8674343f8, 0x005ea77b005ea77b, 0x006e977f006e977f,
            0x8416184284161842, 0xa74343f7a74343f7, 0x305ea77a305ea77a, 0x306e977e306e977e,
            0x8416184384161843, 0xa74343f8a74343f8, 0x305ea77b305ea77b, 0x306e977f306e977f,
            0xf4160842f4160842, 0xc74333f7c74333f7, 0x505e977a505e977a, 0x105e977e105e977e,
            0xf4160843f4160843, 0xc74333f8c74333f8, 0x505e977b505e977b, 0x105e977f105e977f,
            0xf4160842f4160842, 0xc74333f7c74333f7, 0x505e977a505e977a, 0x105e977e105e977e,
            0xf4160843f4160843, 0xc74333f8c74333f8, 0x505e977b505e977b, 0x105e977f105e977f,
            0xf4161842f4161842, 0xc74343f7c74343f7, 0x505ea77a505ea77a, 0x106e977e106e977e,
            0xf4161843f4161843, 0xc74343f8c74343f8, 0x505ea77b505ea77b, 0x106e977f106e977f,
            0xf4161842f4161842, 0xc74343f7c74343f7, 0x505ea77a505ea77a, 0x106e977e106e977e,
            0xf4161843f4161843, 0xc74343f8c74343f8, 0x505ea77b505ea77b, 0x106e977f106e977f,
        ]);
        let b = test::black_box(0xc74343f7c7434ddd);
        time_all(mode, "vec-64x32-pair", data_sets, 10, 20_000_000, |value| {
            let mut x = [0; 64];
            x[63] = value;
            vec::vec64x32_pair(&a, b, &x)
        });
    }

    // Time Vector 64x32
    {
        let a = test::black_box([
            0x9416084294160842, 0x674333f7674333f7, 0x005e977a005e977a, 0x005e977e005e977e,
            0x9416084394160843, 0x674333f8674333f8, 0x005e977b005e977b, 0x005e977f005e977f,
            0x8416084284160842, 0xa74333f7a74333f7, 0x305e977a305e977a, 0x305e977e305e977e,
            0x8416084384160843, 0xa74333f8a74333f8, 0x305e977b305e977b, 0x305e977f305e977f,
            0x9416184294161842, 0x674343f7674343f7, 0x005ea77a005ea77a, 0x006e977e006e977e,
            0x9416184394161843, 0x674343f8674343f8, 0x005ea77b005ea77b, 0x006e977f006e977f,
            0x8416184284161842, 0xa74343f7a74343f7, 0x305ea77a305ea77a, 0x306e977e306e977e,
            0x8416184384161843, 0xa74343f8a74343f8, 0x305ea77b305ea77b, 0x306e977f306e977f,
            0xf4160842f4160842, 0xc74333f7c74333f7, 0x505e977a505e977a, 0x105e977e105e977e,
            0xf4160843f4160843, 0xc74333f8c74333f8, 0x505e977b505e977b, 0x105e977f105e977f,
            0xf4160842f4160842, 0xc74333f7c74333f7, 0x505e977a505e977a, 0x105e977e105e977e,
            0xf4160843f4160843, 0xc74333f8c74333f8, 0x505e977b505e977b, 0x105e977f105e977f,
            0xf4161842f4161842, 0xc74343f7c74343f7, 0x505ea77a505ea77a, 0x106e977e106e977e,
            0xf4161843f4161843, 0xc74343f8c74343f8, 0x505ea77b505ea77b, 0x106e977f106e977f,
            0xf4161842f4161842, 0xc74343f7c74343f7, 0x505ea77a505ea77a, 0x106e977e106e977e,
            0xf4161843f4161843, 0xc74343f8c74343f8, 0x505ea77b505ea77b, 0x106e977f106e977f,
        ]);
        let b = test::black_box(0xc74343f7c7434ddd);
        time_all(mode, "vec-64x32", data_sets, 10, 20_000_000, |value| {
            let mut x = [0; 64];
            x[63] = value;
            vec::vec64x32(&a, b, &x)
        });
    }

    // Time Multiply-Mod-Prime
    {
        let a = test::black_box([0x94160842, 0x674333f7, 0x005e977a]);
        let b = test::black_box([0x55387e64, 0xbdafef1f, 0x008e956a]);

        time_all(mode, "mod-prime", data_sets, 10, 200_000_000, |value| {
            modprime::mod_prime(a, b, [value, 0])
        });
    }

    // Time Multiply-Shift (v1)
    {
        let a = test::black_box(0x94160842);

        time_all(mode, "shift-v1", data_sets, 10, 3000_000_000, |value| {
            shift::shift(a, value as u64)
        });
    }

    // Time SipHash
    {
        let a = test::black_box(0x94160842674333f7);
        let b = test::black_box(0x55387e64bdafef1f);

        time_all(mode, "siphash", data_sets, 10, 100_000_000, |value| {
            #[allow(deprecated)]
            let mut h = SipHasher::new_with_keys(a, b);
            h.write_u32(value);
            h.finish() as u32
        });
    }
}
