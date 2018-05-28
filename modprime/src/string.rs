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
