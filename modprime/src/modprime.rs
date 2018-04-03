#[cfg(test)]
use test;

// Assumptions:
// * The prime is p = 2^89 - 1.
// * The hash size is m = 2^20.

// NOTE: All numbers are represented with the least significant parts first.

// TODO: Rename variable names to be consistent.

pub fn multiply_add(a: [u32; 3], b: [u32; 3], x: [u32; 2]) -> [u32; 5] {
    // Step 1

    // Step 1.1

    let alpha0 = (a[0] as u64) * (x[0] as u64);
    let alpha1 = (a[1] as u64) * (x[0] as u64);
    let alpha2 = (a[2] as u64) * (x[0] as u64);

    let gamma0 = alpha0;
    let gamma1 = (gamma0 >> 32) + alpha1;
    let gamma2 = (gamma1 >> 32) + alpha2;

    let c0 = gamma0 & 0xffffffff;
    let c1 = gamma1 & 0xffffffff;
    let c2 = gamma2 & 0xffffffff;
    let c3 = gamma2 >> 32;

    // Step 1.2

    let alpha0 = c0;
    let alpha1 = c1 + (a[0] as u64) * (x[1] as u64);
    let alpha2 = c2 + (a[1] as u64) * (x[1] as u64);
    let alpha3 = c3 + (a[2] as u64) * (x[1] as u64);

    let gamma0 = alpha0;
    let gamma1 = (gamma0 >> 32) + alpha1;
    let gamma2 = (gamma1 >> 32) + alpha2;
    let gamma3 = (gamma2 >> 32) + alpha3;

    let d0 = gamma0 & 0xffffffff;
    let d1 = gamma1 & 0xffffffff;
    let d2 = gamma2 & 0xffffffff;
    let d3 = gamma3 & 0xffffffff;
    let d4 = gamma3 >> 32;

    // Step 1.3

    let alpha0 = d0 + (b[0] as u64);
    let alpha1 = d1 + (b[1] as u64);
    let alpha2 = d2 + (b[2] as u64);
    let alpha3 = d3;
    let alpha4 = d4;

    let gamma0 = alpha0;
    let gamma1 = (gamma0 >> 32) + alpha1;
    let gamma2 = (gamma1 >> 32) + alpha2;
    let gamma3 = (gamma2 >> 32) + alpha3;
    let gamma4 = (gamma3 >> 32) + alpha4;

    let e0 = gamma0 & 0xffffffff;
    let e1 = gamma1 & 0xffffffff;
    let e2 = gamma2 & 0xffffffff;
    let e3 = gamma3 & 0xffffffff;
    let e4 = gamma4 & 0xffffffff;
    // No e5 part.

    [e0 as u32, e1 as u32, e2 as u32, e3 as u32, e4 as u32]
}

pub fn modulo(e: [u32; 5]) -> u32 {
    // Step 2

    // Step 2.1

    let l0 = e[0]; // 32 bits
    let l1 = e[1]; // 32 bits
    let l2 = e[2] & 0x1ffffff; // 25 bits

    // Step 2.2

    let h0 = (e[2] >> 25) | (e[3] << 7);
    let h1 = (e[3] >> 25) | (e[4] << 7);
    let h2 = e[4] >> 25;

    // Step 2.3

    let alpha0 = (l0 as u64) + (h0 as u64);
    let alpha1 = (l1 as u64) + (h1 as u64);
    let alpha2 = (l2 as u64) + (h2 as u64);

    let gamma0 = alpha0;
    let gamma1 = (gamma0 << 32) + alpha1;
    let gamma2 = (gamma1 << 32) + alpha2;

    let s0 = gamma0 & 0xffffffff;
    let s1 = gamma1 & 0xffffffff;
    let s2 = gamma2 & 0xffffffff;
    // No s3 part.

    // Step 3.1

    let alpha0 = (s0 as i64) - 0xffffffff;
    let alpha1 = (s1 as i64) - 0xffffffff;
    let alpha2 = (s2 as i64) - 0x01ffffff;

    let gamma0 = alpha0;
    let gamma1 = (gamma0 >> 32) + alpha1;
    let gamma2 = (gamma1 >> 32) + alpha2;

    let t0 = (gamma0 as u64) & 0xffffffff;
    let tnonneg = gamma2 >= 0;

    // Step 3.2

    let q0 = if tnonneg { t0 } else { s0 };

    // Step 4.1

    (q0 & 0x000fffff) as u32
}

pub fn mod_prime(a: [u32; 3], b: [u32; 3], x: [u32; 2]) -> u32 {
    // Calculate a x + b.
    let y = multiply_add(a, b, x);
    // Calculate ((a x + b) mod p) mod m.
    let r = modulo(y);
    r
}

// These attributes force the function to be available in the assembly.
#[inline(never)]
#[no_mangle]
pub fn modprime_mod_prime_no_inline(a: [u32; 3], b: [u32; 3], x: [u32; 2]) -> u32 {
    mod_prime(a, b, x)
}

#[test]
fn test_multiply_small() {
    let r = multiply_add([1, 0, 0], [0, 0, 0], [5, 0]);
    assert_eq!([5, 0, 0, 0, 0], r);
}

#[test]
fn test_multiply_big() {
    let r = multiply_add(
        [0x77777777, 0xdddddddd, 0x22222222],
        [0, 0, 0],
        [0xeeeeeeee, 0x33333333],
    );
    assert_eq!(
        [0xd4c3b2a2, 0xcccccccc, 0x56789abb, 0x789abcdf, 0x6d3a06d],
        r
    );
}

#[test]
fn test_multiply_max() {
    let r = multiply_add(
        [0xffffffff, 0xffffffff, 0xffffffff],
        [0, 0, 0],
        [0xffffffff, 0xffffffff],
    );
    assert_eq!([1, 0, 4294967295, 4294967294, 4294967295], r);
}

#[test]
fn test_multiply_add_small() {
    let r = multiply_add([1, 0, 0], [3, 0, 0], [5, 0]);
    assert_eq!([8, 0, 0, 0, 0], r);
}

#[test]
fn test_multiply_add_big() {
    let r = multiply_add(
        [0x77777777, 0xdddddddd, 0x22222222],
        [0x55555555, 0xcccccccc, 0xffffffff],
        [0xeeeeeeee, 0x33333333],
    );
    assert_eq!(
        [706283511, 2576980377, 1450744507, 2023406816, 114532461],
        r
    );
}

#[test]
fn test_multiply_add_max() {
    let r = multiply_add(
        [0xffffffff, 0xffffffff, 0xffffffff],
        [0xffffffff, 0xffffffff, 0xffffffff],
        [0xffffffff, 0xffffffff],
    );
    assert_eq!([0, 0, 0xffffffff, 0xffffffff, 0xffffffff], r);
}

#[test]
fn test_modulo_small() {
    let r = modulo([273, 0, 0, 0, 0]);
    assert_eq!(273, r);
}

#[test]
fn test_modulo_big() {
    let r = modulo([0x77777777, 0x11111111, 0xdddddddd, 0xbbbbbbbb, 0x22222222]);
    assert_eq!(0x55565, r);
}

#[test]
fn test_modulo_max1() {
    let r = modulo([0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff]);
    assert_eq!(0xfffff, r);
}

#[test]
fn test_modulo_max2() {
    // p mod p = 0, so (p mod p) mod m = 0.
    let r = modulo([0xffffffff, 0xffffffff, 0x1ffffff, 0, 0]);
    assert_eq!(0, r);
}

#[test]
fn test_modulo_max3() {
    // (p - 1 mod p) mod m
    let r = modulo([0xfffffffe, 0xffffffff, 0x1ffffff, 0, 0]);
    assert_eq!(0xffffe, r);
}

#[bench]
fn bench(bench: &mut test::Bencher) {
    let a = test::black_box([0x77777777, 0xdddddddd, 0x22222222]);
    let b = test::black_box([0x55555555, 0xcccccccc, 0xffffffff]);
    let x = test::black_box([0xffffffff, 0xffffffff]);

    bench.iter(|| mod_prime(a, b, x));
}
