#[cfg(test)]
use test;

// Assumptions:
// * The universe bit-width is w = 64.
// * The hash bit-width is l = 20.

pub const HASH_WIDTH: u8 = 20;

pub fn shift(a: u64, x: u64) -> u32 {
    (a.wrapping_mul(x) >> (64 - HASH_WIDTH)) as u32
}

// These attributes force the function to be available in the assembly.
#[inline(never)]
#[no_mangle]
pub fn modprime_shift_no_inline(a: u64, x: u64) -> u32 {
    shift(a, x)
}

#[bench]
fn bench(bench: &mut test::Bencher) {
    let a = test::black_box(0x77777777_77777777);
    let x = test::black_box(0xdddddddd_dddddddd);

    bench.iter(|| shift(a, x));
}
