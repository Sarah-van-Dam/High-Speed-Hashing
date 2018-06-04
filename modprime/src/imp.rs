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
fn bench_mod_prime(bench: &mut test::Bencher) {
    let a = test::black_box([0x77777777, 0xdddddddd, 0x22222222]);
    let b = test::black_box([0x55555555, 0xcccccccc, 0xffffffff]);
    let x = test::black_box([0xffffffff, 0xffffffff]);

    bench.iter(|| mod_prime(a, b, x));
}






























/// Implemented: three multiply-mod-prime with p = 2^31 - 1 xor'd together.
/// h_a0,a1,a2,b0,b1,b2 : [2^64] -> [m] for m <= p
pub fn mmp31in64(a: [u32; 3], b: [u32; 3], x: u64) -> u32 {
    let x0 = (x & 0x3fffffff) as u32;
    let x1 = ((x >> 30) & 0x3fffffff) as u32;
    let x2 = ((x >> 60) & 0x3fffffff) as u32;

    let q0 = mmp31(a[0], b[0], x0);
    let q1 = mmp31(a[1], b[1], x1);
    let q2 = mmp31(a[2], b[2], x2);

    q0 ^ q1 ^ q2
}









pub fn mmp31(a: u32, b: u32, x: u32) -> u32 {
    debug_assert!(a < 0x7fffffff);
    debug_assert!(b < 0x7fffffff);
    debug_assert!(x < 0x7fffffff);

    let q = (a as u64) * (x as u64) + (b as u64);
    let s = ((q as u32) & 0x7fffffff) + ((q >> 31) as u32);
    if s >= 0x7fffffff { s - 0x7fffffff } else { s }
}

//pub fn mmp31_in64(a: u32, b: u32, x: u64) -> u32 {
//    let x0 = x & 0x3fffffff;
//    let x1 = (x >> 30) & 0x3fffffff;
//    let x2 = x >> 60;
//}

#[test]
fn test_mmp31() {
    assert_eq!(11_126_486, mmp31(1234, 5678, 9012));
    assert_eq!(0x095df80d, mmp31(0x6d9e78c5, 0x4019affa, 0x770e288c));
}

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
fn bench_shift(bench: &mut test::Bencher) {
    let a = test::black_box(0x77777777_77777777);
    let x = test::black_box(0xdddddddd_dddddddd);

    bench.iter(|| shift(a, x));
}

// Assume w = 32, wbar = 64, l = 20, d = 64.

pub const L: usize = 20;
pub const D: usize = 64;

pub fn prefix64x32_pair(a: &[u64; D + 1], x: &[u32]) -> u32 {
    debug_assert!(x.len() % 2 == 0);
    debug_assert!(x.len() <= D);

    let mut sum = a[x.len()];
    for i in 0..(x.len()/2) {
        let c = a[2*i].wrapping_add(x[2*i + 1] as u64);
        let d = a[2*i + 1].wrapping_add(x[2*i] as u64);
        sum += c.wrapping_mul(d);
    }
    ((sum >> (64 - L)) as u32) & 0xfffff
}

const P16: u64 = (1 << 17) - 1;

pub fn poly16(a: u32, b: u32, c: u32, x: &[u16]) -> u32 {
    debug_assert!(a < P16 as u32);
    debug_assert!(b < P16 as u32);
    debug_assert!(c < P16 as u32);

    if x.is_empty() {
        return b & 0xfffff;
    }

    let mut h = x[0] as u64;
    for &v in &x[1..] {
        h = (c as u64)*h + (v as u64);
        h = (h & 0x1ffff) + (h >> 17);
        // TODO: Probably not optimal.
        if h >= P16 {
            h -= P16;
        }
    }

    h = (a as u64)*h + (b as u64);
    h = (h & 0x1ffff) + (h >> 17);
    if h >= P16 {
        h -= P16;
    }

    (h as u32) & 0xfffff
}

const P64: [u32; 3] = [0xffffffff, 0xffffffff, 0x01ffffff];

#[inline]
pub fn add4x4(a: [u32; 4], b: [u32; 4]) -> [u32; 5] {
    let c0 = (a[0] as u64) + (b[0] as u64);
    let c1 = (a[1] as u64) + (b[1] as u64) + (c0 >> 32);
    let c2 = (a[2] as u64) + (b[2] as u64) + (c1 >> 32);
    let c3 = (a[3] as u64) + (b[3] as u64) + (c2 >> 32);
    let c4 = c3 >> 32;
    [c0 as u32, c1 as u32, c2 as u32, c3 as u32, c4 as u32]
}

#[inline]
pub fn mul3x1(a: [u32; 3], x: u32) -> [u32; 4] {
    let c0 = (a[0] as u64) * (x as u64);
    let c1 = (a[1] as u64) * (x as u64) + (c0 >> 32);
    let c2 = (a[2] as u64) * (x as u64) + (c1 >> 32);
    let c3 = c2 >> 32;
    [c0 as u32, c1 as u32, c2 as u32, c3 as u32]
}

#[inline]
pub fn mul3x1add3(a: [u32; 3], x: u32, b: [u32; 3]) -> [u32; 4] {
    let c0 = (a[0] as u64) * (x as u64) + (b[0] as u64);
    let c1 = (a[1] as u64) * (x as u64) + (b[1] as u64) + (c0 >> 32);
    let c2 = (a[2] as u64) * (x as u64) + (b[2] as u64) + (c1 >> 32);
    let c3 = c2 >> 32;
    [c0 as u32, c1 as u32, c2 as u32, c3 as u32]
}

#[inline]
pub fn mul3x3(a: [u32; 3], x: [u32; 3]) -> [u32; 6] {
    let [d0, c1, c2, c3] = mul3x1(a, x[0]);
    let [d1, c2, c3, c4] = mul3x1add3(a, x[1], [c1, c2, c3]);
    let [d2, d3, d4, d5] = mul3x1add3(a, x[2], [c2, c3, c4]);
    [d0, d1, d2, d3, d4, d5]
}

#[inline]
pub fn mul3x2(a: [u32; 3], x: [u32; 2]) -> [u32; 5] {
    let [d0, c1, c2, c3] = mul3x1(a, x[0]);
    let [d1, d2, d3, d4] = mul3x1add3(a, x[1], [c1, c2, c3]);
    [d0, d1, d2, d3, d4]
}

#[inline]
pub fn splitp6(a: [u32; 6]) -> ([u32; 3], [u32; 3]) {
    let c = [a[0], a[1], a[2] & 0x01ffffff];
    let d = [a[2] >> 25 | a[3] << 7, a[3] >> 25 | a[4] << 7, a[4] >> 25 | a[5] << 7];
    debug_assert_eq!(0, a[5] >> 25);
    (c, d)
}

#[inline]
pub fn add3x3x3_no_overflow(a: [u32; 3], b: [u32; 3], c: [u32; 3]) -> [u32; 3] {
    let d0 = (a[0] as u64) + (b[0] as u64) + (c[0] as u64);
    let d1 = (a[1] as u64) + (b[1] as u64) + (c[1] as u64) + (d0 >> 32);
    let d2 = (a[2] as u64) + (b[2] as u64) + (c[2] as u64) + (d1 >> 32);
    debug_assert_eq!(0, d2 >> 32);
    [d0 as u32, d1 as u32, d2 as u32]
}

#[inline]
pub fn trysubp3(a: [u32; 3]) -> [u32; 3] {
    let c0 = (a[0] as i64) - (P64[0] as i64);
    let c1 = (a[1] as i64) - (P64[1] as i64) + (c0 >> 32);
    let c2 = (a[2] as i64) - (P64[2] as i64) + (c1 >> 32);
    if c2 >= 0 {
        [c0 as u32, c1 as u32, c2 as u32]
    } else {
        a
    }
}

#[inline]
pub fn add6x3modp(a: [u32; 6], b: [u32; 3]) -> [u32; 3] {
    let (c, d) = splitp6(a);
    let e = add3x3x3_no_overflow(c, d, b);
    let f = trysubp3(e);
    let g = trysubp3(f);
    g
}

pub struct Poly64 {
    a: [u32; 3],
    b: [u32; 3],
    c: [u32; 3],
    state: [u32; 3],
}

impl Poly64 {
    #[inline]
    pub fn new(a: [u32; 3], b: [u32; 3], c: [u32; 3]) -> Self {
        let state = [0, 0, 0];
        Self { a, b, c, state }
    }

    #[inline]
    pub fn write_u64(&mut self, x: u64) {
        let x0 = x as u32;
        let x1 = (x >> 32) as u32;
        let s = mul3x3(self.c, self.state);
        self.state = add6x3modp(s, [x0, x1, 0]);
    }

    #[inline]
    pub fn finish(&mut self) -> u64 {
        let s = mul3x3(self.a, self.state);
        let t = add6x3modp(s, self.b);
        self.state = [0, 0, 0];
        ((t[0] as u64) | (t[1] as u64) << 32)
    }
}

pub struct PairPrefixShift32x64 {
    a: [u64; 65],
    i: usize,
    state: u64,
}

impl PairPrefixShift32x64 {
    #[inline]
    pub fn new(a: [u64; 65]) -> Self {
        Self { a, i: 0, state: 0 }
    }

    #[inline]
    pub fn write_u64(&mut self, x: u64) {
        let x0 = x & 0xffffffff;
        let x1 = x >> 32;
        let factor0 = self.a[2*self.i].wrapping_add(x1);
        let factor1 = self.a[2*self.i + 1].wrapping_add(x0);
        let prod = factor0.wrapping_mul(factor1);
        self.state = self.state.wrapping_add(prod);
        self.i += 1;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.i == 32
    }

    #[inline]
    pub fn finish(&mut self) -> u32 {
        let ad = self.a[2*self.i];
        let value = (self.state.wrapping_add(ad) >> 32) as u32;
        self.i = 0;
        self.state = 0;
        value
    }
}

pub struct PolySpeedup64 {
    poly: Poly64,
    prep0: PairPrefixShift32x64,
    prep1: PairPrefixShift32x64,
}

impl PolySpeedup64 {
    #[inline]
    pub fn new(poly: Poly64, prep0: PairPrefixShift32x64, prep1: PairPrefixShift32x64) -> Self {
        Self { poly, prep0, prep1 }
    }

    #[inline]
    pub fn write_u64(&mut self, x: u64) {
        if self.prep0.is_done() {
            self.flush();
        }
        self.prep0.write_u64(x);
        self.prep1.write_u64(x);
    }

    #[inline]
    pub fn finish(&mut self) -> u64 {
        self.flush();
        self.poly.finish()
    }

    #[inline]
    fn flush(&mut self) {
        let q0 = self.prep0.finish();
        let q1 = self.prep1.finish();
        let q = (q0 as u64) | (q1 as u64) << 32;
        self.poly.write_u64(q);
    }
}

#[inline]
pub fn mmp89(a: [u32; 3], b: [u32; 3], x: u64) -> u64 {
    // TODO: Might be less efficient, since we only assume x < p.
    let x = [x as u32, (x >> 32) as u32];
    let [c0, c1, c2, c3, c4] = mul3x2(a, x);
    let d = add6x3modp([c0, c1, c2, c3, c4, 0], b);
    (d[0] as u64) | (d[1] as u64) << 32
}

#[inline]
pub fn shift64(a: u128, b: u128, x: u64) -> u64 {
    (a.wrapping_mul(x as u128).wrapping_add(b) >> 64) as u64
}

#[test]
fn test_poly_speedup_64() {
    let poly = Poly64::new(
        [0xfffffffe, 0xffffffff, 0x01ffffff],
        [0xfffffffe, 0xffffffff, 0x01ffffff],
        [0xfffffffe, 0xffffffff, 0x01ffffff],
    );
    let prep0 = PairPrefixShift32x64::new([0xffffffffffffffff; 65]);
    let prep1 = PairPrefixShift32x64::new([0xffffffffffffffff; 65]);
    let mut h = PolySpeedup64::new(poly, prep0, prep1);
    for &x in &[0xffffffffffffffff; 1024][..] {
        h.write_u64(x);
    }
    // TODO: Check if reasonable.
    assert_eq!(0xfffffffffffffffe, h.finish());
}

pub fn vec64x32(a: &[u64; D], b: u64, x: &[u32; D]) -> u32 {
    let mut sum = b;
    for i in 0..D {
        sum = sum.wrapping_add(a[i].wrapping_mul(x[i] as u64));
    }
    ((sum >> (64 - L)) as u32) & 0xfffff
}

// These attributes force the function to be available in the assembly.
#[inline(never)]
#[no_mangle]
pub fn vec_vec64x32_no_inline(a: &[u64; D], b: u64, x: &[u32; D]) -> u32 {
    vec64x32(a, b, x)
}

pub fn vec64x32_pair(a: &[u64; D], b: u64, x: &[u32; D]) -> u32 {
    let mut sum = b;
    for i in 0..(D/2) {
        let c = a[2*i].wrapping_add(x[2*i + 1] as u64);
        let d = a[2*i + 1].wrapping_add(x[2*i] as u64);
        sum += c.wrapping_mul(d);
    }
    ((sum >> (64 - L)) as u32) & 0xfffff
}

// These attributes force the function to be available in the assembly.
#[inline(never)]
#[no_mangle]
pub fn vec_vec64x32_pair_no_inline(a: &[u64; D], b: u64, x: &[u32; D]) -> u32 {
    vec64x32_pair(a, b, x)
}
