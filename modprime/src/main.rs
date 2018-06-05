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

use byteorder::{ByteOrder, BigEndian};

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
    BigEndian::read_u32_into(input_raw, &mut input_raw_u32[..]);

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

fn time_2<T, F>(mode: OutputMode, rep: u32, num: u32, scheme: &str, input: &[T], func: F)
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
                println!("Scheme: {}; ns/value: {:.6}", scheme, nspervalue);
            }
            OutputMode::Csv => {
                println!("{},{}", scheme, nspervalue);
            }
        }
    }
}

fn experiment_2(mode: OutputMode, input_raw: &[u8]) {
    let input_raw = &input_raw[..input_raw.len() & !255];

    let mut input_32 = Vec::new();
    let mut input_64 = Vec::new();

    for chunk in input_raw.chunks(256) {
        let mut chunk_32 = [0; 64];
        let mut chunk_64 = [0; 32];

        BigEndian::read_u32_into(chunk, &mut chunk_32);
        BigEndian::read_u64_into(chunk, &mut chunk_64);

        input_32.push(chunk_32);
        input_64.push(chunk_64);
    }

    let rep = 10;
    let num = 2000;

    if mode.is_csv() {
        println!("scheme,nspervalue");
    }

    {
        let a = test::black_box([
            0xa32b511bb9419925,
            0x468967dfa5b55d7c,
            0xd42a4cfdeaccd43c,
            0x3c2c20e3f28f94ad,
            0xce58ab0fc65d6b53,
            0xa24f83440516d39e,
            0xfeda02478b1da9bb,
            0x0a1c38a336c2f53f,
            0xee08871d414ea1e4,
            0x751f29778f19e95e,
            0x40714e646dcda33a,
            0xb304dbe1cd04d2ac,
            0x4c58ef616d8f044f,
            0xd006a9b5e0dc2623,
            0x1e9d6de78875186e,
            0x4c7c6c3f07eb6795,
            0x1503435a2323b6de,
            0x697bc32cadc36151,
            0xef2942f4cc29ce0d,
            0x09aadd479d1e4147,
            0x77a506902fc4e94c,
            0x35601d50f726e15c,
            0x359fbdab75f704ec,
            0x08b069380425ddcb,
            0x77071ac116b7bfe2,
            0x0f1fe1f375365ab0,
            0x1df5d02088d82064,
            0x373a6593a7b533dd,
            0xddca0594cabad3fa,
            0xafa30a4218f2473b,
            0xbac3eb0c71667dfd,
            0x73d944d2aeaa1269,
            0x9b3993f6d476ee21,
            0x1d0082cc5add5c6b,
            0xe1c721ec67b9f8d1,
            0x4dd3ae5399c02295,
            0x8c22156793c91933,
            0x539298bdc22fa4d3,
            0x518e460a4cebf181,
            0x28d7214e330ab8f4,
            0x09bc35ac293702d0,
            0x9f2e084081b677bc,
            0x31b981b366dd76e8,
            0xf8411798adebd9f9,
            0x4d7935d75ffa99be,
            0x4a5058b71a2170c3,
            0x16e68a8922ce1dfa,
            0x1b26ad0a35d2745d,
            0x12a7113f0927dca3,
            0x3ebc8b7f8b09920b,
            0xde2de731b4800c4e,
            0x0897fa405ec8cca4,
            0xf839242a1cda43c5,
            0xfb8b84894d9d4947,
            0x392d27343c4f233d,
            0x9d606bb797002b7f,
            0x8ee61bbd1967b081,
            0xfc60e7cd5f76f82c,
            0x91cbb2891a10527a,
            0xcfd9ead6bbe3b6c8,
            0xc091e4ed4cf36d45,
            0xf44ac339e3d30263,
            0x7dfecb5fda4973ad,
            0xc176a7a265736f18,
            0xa8ce3b04fbd2e1d3,
        ]);
        time_2(mode, rep, num, "vector-shift", &input_32[..], |input, tmp| {
            for chunk in input {
                let mut h = imp::VectorShiftU32D64::new(a);
                for &value in &chunk[..] {
                    h.write_u32(value);
                }
                *tmp ^= h.finish(20);
            }
        });
    }
    {
        let a = test::black_box([
            0x63b92c3f6df33488,
            0xa2207bc53adff964,
            0xaec2dca88ddb1e71,
            0x79e19e87fa120cd8,
            0x85bd7747f7d1493f,
            0xfdde9f44942c2df6,
            0x7ef56cb9766c7bc6,
            0xdec9e842facb5ba5,
            0x46f60b65eb67e0cc,
            0x6997be6404ca980b,
            0x8c0dbf8d1edc9e70,
            0xe49f1e9651cd3d49,
            0x58f51b8593abac4a,
            0x1c2aa379835987a3,
            0x92436454c98acd6f,
            0xf15831f1005edb4b,
            0xce7e34a170f36676,
            0x8e0684e862399c99,
            0x44aa63cc74e9d839,
            0x1ca6f7fdc88bd05b,
            0x1e1b5d120571c33f,
            0xa6e3aad03a3ff85f,
            0x8ae600281f7019d9,
            0x8559f1287c8bdaf7,
            0xdf63429a69a3b57b,
            0x4a4b9478b5a14152,
            0xaa94a0588831596f,
            0x21e11fe26c62825b,
            0x7a5392426a883bdb,
            0x2f3dcaa473d8222b,
            0xebe8bfe52c1bbca7,
            0xa5783fcb6379c163,
            0x4aa81d8a54b34ff4,
            0xefedb73f6cba9878,
            0x56182b429bc1d9f3,
            0xfa1ea680e285753f,
            0x21883ba4784126a2,
            0x52a07eee2741d3c9,
            0x322a1706fbcd82e7,
            0xec396c42fe6c180d,
            0x5c4819bf3abf0554,
            0x02331e406dc18024,
            0xd8c4e83242264407,
            0x51652714bd893042,
            0x7d5839310d07efe3,
            0xaf6f52fe3072a8f8,
            0x1c4fc232dd6ff784,
            0x9c9d5a588e373486,
            0x6962398ddf75a9e4,
            0x6dfe23682b96a306,
            0x02a9ffbcf9a2549f,
            0x171495363537cbb4,
            0x0e33ee376f9c6c7a,
            0x7e866b96944047b9,
            0x59ec7828c2c71193,
            0x93c8b1adc117d060,
            0x92b721f7bbf2b356,
            0xe14f685a4a6e66b7,
            0xda4eeef5ca762528,
            0xee00e49829361caa,
            0xba45113f9d873b85,
            0x0dfad687c9ebd0b8,
            0x97d359cd16140081,
            0x1d53acb7d0ce02aa,
            0xc6f31d83502a08d0,
        ]);
        time_2(mode, rep, num, "pair-shift", &input_64[..], |input, tmp| {
            for chunk in input {
                let mut h = imp::PairShiftU64D32::new(a);
                for &value in &chunk[..] {
                    h.write_u64(value);
                }
                *tmp ^= h.finish(20) as u32;
            }
        });
    }
}

fn experiment_3(mode: OutputMode, input_raw: &[u8]) {
    let input_8 = &input_raw[..input_raw.len() & !7];

    let mut input_64 = vec![0; input_8.len() / 8];

    BigEndian::read_u64_into(&input_8, &mut input_64[..]);

    let rep = 10;
    let num = 400;

    {
        let a = test::black_box([1, 0, 0]);
        let b = test::black_box([2, 0, 0]);
        let c = test::black_box([3, 0, 0]);
        time_2(mode, rep, num, "poly-64", &input_64[..], |input, tmp| {
            let mut h = imp::PolyU64::new(a, b, c);
            for &value in input {
                h.write_u64(value);
            }
            *tmp ^= h.finish(20) as u32;
        });
    }
    {
        let prep1 = test::black_box([
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
        ]);
        let prep2 = test::black_box([
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
        ]);
        let a = test::black_box([1, 0, 0]);
        let b = test::black_box([2, 0, 0]);
        let c = test::black_box([3, 0, 0]);
        time_2(mode, rep, num, "poly-64", &input_64[..], |input, tmp| {
            let h0 = imp::PolyU64::new(a, b, c);
            let h1 = imp::PairShiftU64D32::new(prep1);
            let h2 = imp::PairShiftU64D32::new(prep2);
            let mut h = imp::PreprocPolyU64D32::new(h0, h1, h2);
            for &value in input {
                h.write_u64(value);
            }
            *tmp ^= h.finish(20) as u32;
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

    let num = 3;

    match num {
        1 => experiment_1(mode, &input_raw),
        2 => experiment_2(mode, &input_raw),
        3 => experiment_3(mode, &input_raw),
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
                let mut h = imp::PairShiftU64D32::new(a);
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
                let prep0 = imp::PairShiftU64D32::new(a0);
                let prep1 = imp::PairShiftU64D32::new(a1);
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
