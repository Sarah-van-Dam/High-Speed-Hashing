#![feature(test, sip_hash_13)]

extern crate byteorder;
extern crate test;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process;
use std::time::Instant;

#[allow(deprecated)]
use std::hash::{Hasher, SipHasher};

use byteorder::ByteOrder;

pub mod imp;

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

pub fn time_string<T, F>(input: &[T], n: u32, func: F) -> f64
where
    F: Fn(&[T]) -> u64,
{
    let start = Instant::now();

    let mut tmp = 0;
    for _ in 0..n {
        tmp ^= func(input);
    }

    let elapsed = start.elapsed();

    let _ = test::black_box(tmp);

    let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1e9);
    secs
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

pub fn time_all_string<T, F>(
    mode: OutputMode,
    func_name: &str,
    data_sets: &[(&str, &[T])],
    num_reps: u32,
    target_reps: u32,
    func: F,
) where
    F: Fn(&[T]) -> u64,
{
    for &(data_set_name, data_set) in data_sets {
        let num_values = data_set.len() as u32;
        let n = (target_reps + num_values - 1) / num_values;
        let actual_reps = n * num_values;

        // Run through once to warm up the caches.
        time_string(data_set, 1, &func);

        for _ in 0..num_reps {
            let secs = time_string(data_set, n, &func);
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

fn time_1<T, F>(mode: OutputMode, rep: u32, num: u32, scheme: &str, bits: u32, is_128: bool, input: &[T], func: F)
where
    F: Fn(&[T], &mut u32),
{
    let input = test::black_box(input);

    for _ in 0..rep {
        let mut tmp = 0;

        let start = Instant::now();
        for _ in 0..num {
            func(input, &mut tmp);
        }
        let elapsed = start.elapsed();

        let _ = test::black_box(tmp);

        let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1e9);
        let nspervalue = secs / (num as f64) / (input.len() as f64) * 1e9;

        match mode {
            OutputMode::Pretty => {
                println!("Scheme: {}, input bit-length: {}, 128-bit: {}; ns/value: {:.6}", scheme, bits, is_128, nspervalue);
            }
            OutputMode::Csv => {
                let is_128 = if is_128 { "TRUE" } else { "FALSE" };
                println!("{},{},{},{}", scheme, bits, is_128, nspervalue);
            }
        }
    }
}


fn experiment_1(mode: OutputMode, input_raw: &[u8]) {
    let input_raw = &input_raw[..input_raw.len() & !3];

    let mut input_raw_u32 = vec![0; input_raw.len() / 4];
    byteorder::BigEndian::read_u32_into(input_raw, &mut input_raw_u32[..]);

    let input_32 = input_raw_u32.iter().map(|&value| value & 0x3fffffff).collect::<Vec<_>>();
    let input_64 = input_32.iter().map(|&value| u64::from(value)).collect::<Vec<_>>();
    let input_128 = input_32.iter().map(|&value| u128::from(value)).collect::<Vec<_>>();

    let rep = 10;
    let num = 1000;

    if mode.is_csv() {
        println!("scheme,bits,is128,nspervalue");
    }

    // Shift

    {
        let a = test::black_box(0x3bca40c7);
        time_1(mode, rep, num, "shift", 32, false, &input_32[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::shift_u32(20, a, value);
            }
        });
    }
    {
        let a = test::black_box(0xa570f20b9bd5adfb);
        time_1(mode, rep, num, "shift", 64, false, &input_64[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::shift_u64(20, a, value) as u32;
            }
        });
    }
    {
        let a = test::black_box(0x2cb56e50f9538749b4a1648382ba0d59);
        time_1(mode, rep, num, "shift", 128, true, &input_128[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::shift_u128_128(20, a, value) as u32;
            }
        });
    }
    {
        let a = test::black_box(0x9cb37f1a);
        let b = test::black_box(0x2d8b1736);
        time_1(mode, rep, num, "shift-strong", 32, false, &input_32[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::shift_strong_u32(20, a, b, value) as u32;
            }
        });
    }
    {
        let a = test::black_box(0x6865db19e3d1b464);
        let b = test::black_box(0x583bc159d427a991);
        time_1(mode, rep, num, "shift-strong", 64, true, &input_64[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::shift_strong_u64_128(20, a, b, value) as u32;
            }
        });
    }

    // Multiply-Mod-Prime

    {
        let a = test::black_box(0x40ed8147);
        let b = test::black_box(0x64b07a26);
        time_1(mode, rep, num, "mmp", 30, false, &input_32[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::mmp_p31_u30(20, a, b, value);
            }
        });
    }
    {
        let a = test::black_box([0x68dc5b2d, 0x29ad0bce, 0x278a331a]);
        let b = test::black_box([0x3e4f5b23, 0x2e47ea16, 0x3c665bad]);
        time_1(mode, rep, num, "mmp-triple", 64, false, &input_64[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::mmp_p31_u64(20, a, b, value);
            }
        });
    }
    {
        let a = test::black_box(0x02f52fcd0b6474c3);
        let b = test::black_box(0x0cb11e6766f6e421);
        time_1(mode, rep, num, "mmp", 60, true, &input_32[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::mmp_p61_u60_128(20, a, b, u64::from(value)) as u32;
            }
        });
    }
    {
        let a = test::black_box([0xc543be39, 0xf663c8a4, 0x017193ad]);
        let b = test::black_box([0x180375ec, 0xd6fbb57d, 0x0010c0af]);
        time_1(mode, rep, num, "mmp", 64, false, &input_64[..], |input, tmp| {
            for &value in input {
                *tmp ^= imp::mmp_p89_u64(20, a, b, value) as u32;
            }
        });
    }
}

fn main() {
    let mut input_raw = Vec::new();
    {
        let mut file = File::open("input.txt").unwrap();
        file.read_to_end(&mut input_raw).unwrap();
    }

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

    let num = 1;

    match num {
        1 => experiment_1(mode, &input_raw),
        _ => panic!(),
    }

    /*

    panic!();


    // Multiply-Shift

    {
        let a = test::black_box(0x9416084294160842);

        time_all(mode, "shift-u64", data_sets, 10, 100_000_000, |value| {
            imp::shift_u64(20, a, u64::from(value)) as u32
        });
    }
    {
        let a = test::black_box(0x94160842);

        time_all(mode, "shift-u32", data_sets, 10, 100_000_000, |value| {
            imp::shift_u32(20, a, value)
        });
    }
    {
        let a = test::black_box(0x9416084294160842674333f7674333f7);

        time_all(
            mode,
            "shift-u128-128",
            data_sets,
            10,
            100_000_000,
            |value| imp::shift_u128_128(20, a, u128::from(value)) as u32,
        );
    }
    {
        let a = test::black_box(0x941608429416084);
        let b = test::black_box(0x505e977b505e977);

        time_all(
            mode,
            "shift-strong-u32",
            data_sets,
            10,
            100_000_000,
            |value| imp::shift_strong_u32(20, a, b, value),
        );
    }
    {
        let a = test::black_box(0x9416084294160842674333f7674333f7);
        let b = test::black_box(0x505e977b505e977b105e977f105e977f);

        time_all(
            mode,
            "shift-strong-u64-128",
            data_sets,
            10,
            100_000_000,
            |value| imp::shift_strong_u64_128(20, a, b, u64::from(value)) as u32,
        );
    }

    // Multiply-Mod-Prime

    {
        let a = test::black_box([0x94160842, 0x674333f7, 0x005e977a]);
        let b = test::black_box([0x55387e64, 0xbdafef1f, 0x008e956a]);

        time_all(mode, "mmp-p89-u64", data_sets, 10, 10_000_000, |value| {
            imp::mmp_p89_u64(20, a, b, u64::from(value)) as u32
        });
    }
    {
        let a = test::black_box(0x674333f7);
        let b = test::black_box(0xbdafef1f);

        time_all(mode, "mmp-p31-u30", data_sets, 10, 10_000_000, |value| {
            imp::mmp_p31_u30(20, a, b, value & 0x3fffffff)
        });
    }
    {
        let a = test::black_box([0x39410842, 0x674333f7, 0x005e977a]);
        let b = test::black_box([0x3d587e64, 0xbdafef1f, 0x008e956a]);

        time_all(mode, "mmp-p31-u64", data_sets, 10, 10_000_000, |value| {
            imp::mmp_p31_u64(20, a, b, u64::from(value))
        });
    }
    {
        let a = test::black_box(0x674333f7005e977a);
        let b = test::black_box(0xbdafef1f008e956a);

        time_all(
            mode,
            "mmp-p61-u60-128",
            data_sets,
            10,
            10_000_000,
            |value| imp::mmp_p61_u60_128(20, a, b, u64::from(value)) as u32,
        );
    }

    // Vectorized

    {
        let a = test::black_box([
            0x9416084294160842,
            0x674333f7674333f7,
            0x005e977a005e977a,
            0x005e977e005e977e,
            0x9416084394160843,
            0x674333f8674333f8,
            0x005e977b005e977b,
            0x005e977f005e977f,
            0x8416084284160842,
            0xa74333f7a74333f7,
            0x305e977a305e977a,
            0x305e977e305e977e,
            0x8416084384160843,
            0xa74333f8a74333f8,
            0x305e977b305e977b,
            0x305e977f305e977f,
            0x9416184294161842,
            0x674343f7674343f7,
            0x005ea77a005ea77a,
            0x006e977e006e977e,
            0x9416184394161843,
            0x674343f8674343f8,
            0x005ea77b005ea77b,
            0x006e977f006e977f,
            0x8416184284161842,
            0xa74343f7a74343f7,
            0x305ea77a305ea77a,
            0x306e977e306e977e,
            0x8416184384161843,
            0xa74343f8a74343f8,
            0x305ea77b305ea77b,
            0x306e977f306e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xc74343f7c7434ddd,
        ]);

        time_all(
            mode,
            "vector-shift-u32-d64",
            data_sets,
            10,
            10_000_000,
            |value| {
                let mut buf = [0; 29];
                buf[3] = value;
                let mut h = imp::VectorShiftU32D64::new(a);
                for &x in &buf[..] {
                    h.write_u32(x);
                }
                h.finish(20)
            },
        );
    }
    {
        let a = test::black_box([
            0x9416084294160842,
            0x674333f7674333f7,
            0x005e977a005e977a,
            0x005e977e005e977e,
            0x9416084394160843,
            0x674333f8674333f8,
            0x005e977b005e977b,
            0x005e977f005e977f,
            0x8416084284160842,
            0xa74333f7a74333f7,
            0x305e977a305e977a,
            0x305e977e305e977e,
            0x8416084384160843,
            0xa74333f8a74333f8,
            0x305e977b305e977b,
            0x305e977f305e977f,
            0x9416184294161842,
            0x674343f7674343f7,
            0x005ea77a005ea77a,
            0x006e977e006e977e,
            0x9416184394161843,
            0x674343f8674343f8,
            0x005ea77b005ea77b,
            0x006e977f006e977f,
            0x8416184284161842,
            0xa74343f7a74343f7,
            0x305ea77a305ea77a,
            0x306e977e306e977e,
            0x8416184384161843,
            0xa74343f8a74343f8,
            0x305ea77b305ea77b,
            0x306e977f306e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xc74343f7c7434ddd,
        ]);

        time_all(
            mode,
            "pair-prefix-shift-u64-d32",
            data_sets,
            10,
            10_000_000,
            |value| {
                let mut buf = [0; 29];
                buf[3] = value;
                let mut h = imp::PairPrefixShiftU64D32::new(a);
                for &x in &buf[..] {
                    h.write_u64(u64::from(x));
                }
                h.finish(20)
            },
        );
    }

    // Polynomial

    {
        let a = test::black_box([0x94160842, 0x674333f7, 0x005e977a]);
        let b = test::black_box([0x55387e64, 0xbdafef1f, 0x008e956a]);
        let c = test::black_box([0x55347e64, 0xb12fef1f, 0x0085951a]);

        time_all_string(mode, "poly-u64", data_sets, 10, 20_000_000, |buf| {
            let mut h = imp::PolyU64::new(a, b, c);
            for &x in buf {
                h.write_u64(u64::from(x));
            }
            h.finish(20)
        })
    }
    {
        let a0 = test::black_box([
            0x9416084294160842,
            0x674333f7674333f7,
            0x005e977a005e977a,
            0x005e977e005e977e,
            0x9416084394160843,
            0x674333f8674333f8,
            0x005e977b005e977b,
            0x005e977f005e977f,
            0x8416084284160842,
            0xa74333f7a74333f7,
            0x305e977a305e977a,
            0x305e977e305e977e,
            0x8416084384160843,
            0xa74333f8a74333f8,
            0x305e977b305e977b,
            0x305e977f305e977f,
            0x9416184294161842,
            0x674343f7674343f7,
            0x005ea77a005ea77a,
            0x006e977e006e977e,
            0x9416184394161843,
            0x674343f8674343f8,
            0x005ea77b005ea77b,
            0x006e977f006e977f,
            0x8416184284161842,
            0xa74343f7a74343f7,
            0x305ea77a305ea77a,
            0x306e977e306e977e,
            0x8416184384161843,
            0xa74343f8a74343f8,
            0x305ea77b305ea77b,
            0x306e977f306e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xc74343f7c7434ddd,
        ]);
        let a1 = test::black_box([
            0x8416084294160842,
            0x674333f7674333f7,
            0x005e977a005e977a,
            0x005e977e005e977e,
            0x9416084394160843,
            0x674333f8674333f8,
            0x005e977b005e977b,
            0x005e977f005e977f,
            0x8416084284160842,
            0xa74333f7a74333f7,
            0x305e977a305e977a,
            0x305e977e305e977e,
            0x8416084384160843,
            0xa74333f8a74333f8,
            0x305e977b305e977b,
            0x305e977f305e977f,
            0x9416184294161842,
            0x674343f7674343f7,
            0x005ea77a005ea77a,
            0x006e977e006e977e,
            0x9416184394161843,
            0x674343f8674343f8,
            0x005ea77b005ea77b,
            0x006e977f006e977f,
            0x8416184284161842,
            0xa74343f7a74343f7,
            0x305ea77a305ea77a,
            0x306e977e306e977e,
            0x8416184384161843,
            0xa74343f8a74343f8,
            0x305ea77b305ea77b,
            0x306e977f306e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4160842f4160842,
            0xc74333f7c74333f7,
            0x505e977a505e977a,
            0x105e977e105e977e,
            0xf4160843f4160843,
            0xc74333f8c74333f8,
            0x505e977b505e977b,
            0x105e977f105e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xf4161842f4161842,
            0xc74343f7c74343f7,
            0x505ea77a505ea77a,
            0x106e977e106e977e,
            0xf4161843f4161843,
            0xc74343f8c74343f8,
            0x505ea77b505ea77b,
            0x106e977f106e977f,
            0xc74343f7c7434ddd,
        ]);
        let a = test::black_box([0x94160842, 0x674333f7, 0x005e977a]);
        let b = test::black_box([0x55387e64, 0xbdafef1f, 0x008e956a]);
        let c = test::black_box([0x55347e64, 0xb12fef1f, 0x0085951a]);

        time_all_string(
            mode,
            "preproc-poly-u64-d32",
            data_sets,
            10,
            20_000_000,
            |buf| {
                let poly = imp::PolyU64::new(a, b, c);
                let prep0 = imp::PairPrefixShiftU64D32::new(a0);
                let prep1 = imp::PairPrefixShiftU64D32::new(a1);
                let mut h = imp::PreprocPolyU64D32::new(poly, prep0, prep1);
                for &x in buf {
                    h.write_u64(u64::from(x));
                }
                h.finish(20)
            },
        )
    }

    // SipHash

    {
        let a = test::black_box(0x94160842674333f7);
        let b = test::black_box(0x55387e64bdafef1f);

        time_all(mode, "siphash", data_sets, 10, 10_000_000, |value| {
            #[allow(deprecated)]
            let mut h = SipHasher::new_with_keys(a, b);
            h.write_u32(value);
            (h.finish() & ((1 << 20) - 1)) as u32
        });
    }

    */
}
