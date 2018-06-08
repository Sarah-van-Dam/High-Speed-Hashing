#[cfg(test)]
use test;

////////////////////////////////////////
// Multiply-Mod-Prime
////////////////////////////////////////

pub const M31: u32 = 0x7fffffff;
pub const M61: u64 = 0x1fffffff_ffffffff;
pub const M89: [u32; 3] = [0xffffffff, 0xffffffff, 0x01ffffff];

// Constants: p = 2^89 - 1
// Interface: u = 2^64, m = 2^l, l <= 64
// Parameters: a, b < p

#[inline]
pub fn mmp_p89_u64(l: usize, a: [u32; 3], b: [u32; 3], x: u64) -> u64 {
    debug_assert!(l <= 64);

    let x = [x as u32, (x >> 32) as u32];
    let [c0, c1, c2, c3, c4] = mul3x2(a, x);
    let d = add6x3modp([c0, c1, c2, c3, c4, 0], b);
    let q = (d[0] as u64) | (d[1] as u64) << 32;

    q & ((1 << l) - 1)
}

// Constants: p = 2^31 - 1
// Interface: u <= p, m = 2^l, l <= 32
// Parameters: a, b < p

#[inline]
pub fn mmp_p31_u30(l: usize, a: u32, b: u32, x: u32) -> u32 {
    debug_assert!(l <= 30);
    debug_assert!(a < M31);
    debug_assert!(b < M31);
    debug_assert!(x < M31);

    let r = u64::from(a) * u64::from(x) + u64::from(b);
    let s = ((r as u32) & M31) + ((r >> 31) as u32);
    let q = if s >= M31 { s - M31 } else { s };

    q & ((1 << l) - 1)
}

// Description: Three independent mmp31 xor'd together.
// Constants: p = 2^31 - 1
// Interface: u = 2^64, m = 2^l, l < 31
// Parameters: a0, a1, a2, b0, b1, b2 < p

#[inline]
pub fn mmp_p31_u64(l: usize, a: [u32; 3], b: [u32; 3], x: u64) -> u32 {
    let x0 = (x & 0x3fffffff) as u32;
    let x1 = ((x >> 30) & 0x3fffffff) as u32;
    let x2 = ((x >> 60) & 0x3fffffff) as u32;

    let q0 = mmp_p31_u30(l, a[0], b[0], x0);
    let q1 = mmp_p31_u30(l, a[1], b[1], x1);
    let q2 = mmp_p31_u30(l, a[2], b[2], x2);

    q0 ^ q1 ^ q2
}

// Constants: p = 2^61 - 1
// Interface: u <= p, m = 2^l, l < 61
// Parameters: a, b < p

#[inline]
pub fn mmp_p61_u60_128(l: usize, a: u64, b: u64, x: u64) -> u64 {
    debug_assert!(l <= 60);
    debug_assert!(a < M61);
    debug_assert!(b < M61);
    debug_assert!(x < M61);

    let r = u128::from(a) * u128::from(x) + u128::from(b);
    let s = ((r as u64) & M61) + ((r >> 61) as u64);
    let q = if s >= M61 { s - M61 } else { s };

    q & ((1 << l) - 1)
}

////////////////////////////////////////
// Multiply-Shift
////////////////////////////////////////

#[inline]
pub fn shift_u32(l: usize, a: u32, x: u32) -> u32 {
    debug_assert!(l <= 32);
    a.wrapping_mul(x) >> (32 - l)
}

#[inline]
pub fn shift_u64(l: usize, a: u64, x: u64) -> u64 {
    debug_assert!(l <= 64);
    a.wrapping_mul(x) >> (64 - l)
}

#[inline]
pub fn shift_u128_128(l: usize, a: u128, x: u128) -> u128 {
    debug_assert!(l <= 128);
    a.wrapping_mul(x) >> (128 - l)
}

#[inline]
pub fn shift_strong_u32(l: usize, a: u64, b: u64, x: u32) -> u32 {
    debug_assert!(l <= 32);
    (a.wrapping_mul(u64::from(x)).wrapping_add(b) >> (64 - l)) as u32
}

#[inline]
pub fn shift_strong_u64_128(l: usize, a: u128, b: u128, x: u64) -> u64 {
    debug_assert!(l <= 64);
    (a.wrapping_mul(u128::from(x)).wrapping_add(b) >> (128 - l)) as u64
}

////////////////////////////////////////
// Vectorized Multiply-Shift
////////////////////////////////////////

// Interface: u = 2^32, d = 64, m = 2^l, l <= 32
// Parameters: a[i] < 2^32

pub struct VectorShiftU32D64 {
    a: [u64; 65],
    i: usize,
    state: u64,
}

impl VectorShiftU32D64 {
    #[inline]
    pub fn new(a: [u64; 65]) -> Self {
        Self { a, i: 0, state: 0 }
    }

    #[inline]
    pub fn write_u32(&mut self, x: u32) {
        let prod = self.a[self.i].wrapping_mul(u64::from(x));
        self.state = self.state.wrapping_add(prod);
        self.i += 1;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.i == 64
    }

    #[inline]
    pub fn finish(&mut self, l: usize) -> u32 {
        debug_assert!(l <= 32);
        let value = (self.state.wrapping_add(self.a[self.i]) >> (64 - l)) as u32;
        self.i = 0;
        self.state = 0;
        value
    }
}

// Interface: u = 2^64, d = 32, m = 2^l, l <= 32
// Parameters: a[i] < 2^64

pub struct PairShiftU64D32 {
    a: [u64; 65],
    i: usize,
    state: u64,
}

impl PairShiftU64D32 {
    #[inline]
    pub fn new(a: [u64; 65]) -> Self {
        Self { a, i: 0, state: 0 }
    }

    #[inline]
    pub fn write_u64(&mut self, x: u64) {
        let x0 = x & 0xffffffff;
        let x1 = x >> 32;
        let factor0 = self.a[2 * self.i].wrapping_add(x1);
        let factor1 = self.a[2 * self.i + 1].wrapping_add(x0);
        let prod = factor0.wrapping_mul(factor1);
        self.state = self.state.wrapping_add(prod);
        self.i += 1;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.i == 32
    }

    #[inline]
    pub fn finish(&mut self, l: usize) -> u32 {
        debug_assert!(l <= 32);
        let ad = self.a[2 * self.i];
        let value = (self.state.wrapping_add(ad) >> (64 - l)) as u32;
        self.i = 0;
        self.state = 0;
        value
    }
}

////////////////////////////////////////
// Polynomial
////////////////////////////////////////

// Constants: p = 2^89 - 1
// Interface: u = 2^64, m = 2^l, l <= 64
// Parameters: a, b, c < p

pub struct PolyU64 {
    a: [u32; 3],
    b: [u32; 3],
    c: [u32; 3],
    state: [u32; 3],
}

impl PolyU64 {
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
    pub fn finish(&mut self, l: usize) -> u64 {
        debug_assert!(l <= 64);
        let s = mul3x3(self.a, self.state);
        let t = add6x3modp(s, self.b);
        let value = (t[0] as u64) | ((t[1] as u64) << 32);
        self.state = [0, 0, 0];
        value & ((1 << l) - 1)
    }
}

// Constants: p = 2^89 - 1
// Interface: u = 2^64, m = 2^l, l <= 32
// Parameters: c < p; a[i], b[i] < 2^64

pub struct PolyShiftU64 {
    a: [u64; 3],
    b: [u64; 3],
    c: [u32; 3],
    state: [u32; 3],
}

impl PolyShiftU64 {
    #[inline]
    pub fn new(a: [u64; 3], b: [u64; 3], c: [u32; 3]) -> Self {
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
    pub fn finish(&mut self, l: usize) -> u32 {
        let q0 = shift_strong_u32(l, self.a[0], self.b[0], self.state[0]);
        let q1 = shift_strong_u32(l, self.a[1], self.b[1], self.state[1]);
        let q2 = shift_strong_u32(l, self.a[2], self.b[2], self.state[2]);

        self.state = [0, 0, 0];

        q0 ^ q1 ^ q2
    }
}

// Constants: p = 2^89 - 1, d = 32
// Interface: u = 2^64, m = 2^l, l <= 64
// Parameters: prep1[i], prep2[i] < 2^64; a, b, c < p

pub struct PreprocPolyU64D32 {
    prep1: PairShiftU64D32,
    prep2: PairShiftU64D32,
    poly: PolyU64,
}

impl PreprocPolyU64D32 {
    #[inline]
    pub fn new(prep1: [u64; 65], prep2: [u64; 65], a: [u32; 3], b: [u32; 3], c: [u32; 3]) -> Self {
        Self {
            prep1: PairShiftU64D32::new(prep1),
            prep2: PairShiftU64D32::new(prep2),
            poly: PolyU64::new(a, b, c),
        }
    }

    #[inline]
    pub fn write_u64(&mut self, x: u64) {
        if self.prep1.is_done() {
            self.flush();
        }
        self.prep1.write_u64(x);
        self.prep2.write_u64(x);
    }

    #[inline]
    pub fn finish(&mut self, l: usize) -> u64 {
        self.flush();
        self.poly.finish(l)
    }

    #[inline]
    fn flush(&mut self) {
        let q1 = self.prep1.finish(32);
        let q2 = self.prep2.finish(32);
        let q = (q1 as u64) | (q2 as u64) << 32;
        self.poly.write_u64(q);
    }
}

////////////////////////////////////////
// Helper Functions
////////////////////////////////////////

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
    let d = [
        a[2] >> 25 | a[3] << 7,
        a[3] >> 25 | a[4] << 7,
        a[4] >> 25 | a[5] << 7,
    ];
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
    let c0 = (a[0] as i64) - (M89[0] as i64);
    let c1 = (a[1] as i64) - (M89[1] as i64) + (c0 >> 32);
    let c2 = (a[2] as i64) - (M89[2] as i64) + (c1 >> 32);
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

////////////////////////////////////////
// Tests and Micro Benchmarks
////////////////////////////////////////

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

#[test]
fn test_mmp31() {
    assert_eq!(11_126_486, mmp31(1234, 5678, 9012));
    assert_eq!(0x095df80d, mmp31(0x6d9e78c5, 0x4019affa, 0x770e288c));
}

#[bench]
fn bench_shift(bench: &mut test::Bencher) {
    let a = test::black_box(0x77777777_77777777);
    let x = test::black_box(0xdddddddd_dddddddd);

    bench.iter(|| shift(a, x));
}

#[test]
fn test_poly_speedup_64() {
    let poly = Poly64::new(
        [0xfffffffe, 0xffffffff, 0x01ffffff],
        [0xfffffffe, 0xffffffff, 0x01ffffff],
        [0xfffffffe, 0xffffffff, 0x01ffffff],
    );
    let prep0 = PairShift32x64::new([0xffffffffffffffff; 65]);
    let prep1 = PairShift32x64::new([0xffffffffffffffff; 65]);
    let mut h = PolySpeedup64::new(poly, prep0, prep1);
    for &x in &[0xffffffffffffffff; 1024][..] {
        h.write_u64(x);
    }
    // TODO: Check if reasonable.
    assert_eq!(0xfffffffffffffffe, h.finish());
}
