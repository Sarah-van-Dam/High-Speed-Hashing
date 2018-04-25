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
pub fn op3_mul1(a: [u32; 3], x: u32) -> [u32; 4] {
    let b0 = (a[0] as u64) * (x as u64);
    let b1 = (a[1] as u64) * (x as u64);
    let b2 = (a[2] as u64) * (x as u64);

    let c0 = b0;
    let c1 = (c0 >> 32) + b1;
    let c2 = (c1 >> 32) + b2;
    let c3 = c2 >> 32;

    [c0 as u32, c1 as u32, c2 as u32, c3 as u32]
}

#[inline]
pub fn op3_mul1_add3(a: [u32; 3], x: u32, b: [u32; 3]) -> [u32; 4] {
    let c0 = (a[0] as u64) * (x as u64) + (b[0] as u64);
    let c1 = (a[1] as u64) * (x as u64) + (b[1] as u64);
    let c2 = (a[2] as u64) * (x as u64) + (b[2] as u64);

    let d0 = c0;
    let d1 = (d0 >> 32) + c1;
    let d2 = (d1 >> 32) + c2;
    let d3 = d2 >> 32;

    [d0 as u32, d1 as u32, d2 as u32, d3 as u32]
}

#[inline]
pub fn op3_mul3(a: [u32; 3], x: [u32; 3]) -> [u32; 6] {
    let [r0, b1, b2, b3] = op3_mul1(a, x[0]);
    let [r1, c2, c3, c4] = op3_mul1_add3(a, x[1], [b1, b2, b3]);
    let [r2, r3, r4, r5] = op3_mul1_add3(a, x[2], [c2, c3, c4]);
    [r0, r1, r2, r3, r4, r5]
}

#[inline]
pub fn op2p_digitsum_add1p(a: [u32; 6], b: [u32; 3]) -> [u32; 3] {
    let c0 = a[0];
    let c1 = a[1];
    let c2 = a[2] & 0x01ffffff;

    let d0 = (a[2] >> 25) | (a[3] << 7);
    let d1 = (a[3] >> 25) | (a[4] << 7);
    let d2 = (a[4] >> 25) | (a[5] << 7);

    let e0 = (c0 as u64) + (d0 as u64) + (b[0] as u64);
    let e1 = (e0 >> 32) + (c1 as u64) + (d1 as u64) + (b[1] as u64);
    let e2 = (e1 >> 32) + (c2 as u64) + (d2 as u64) + (b[2] as u64);

    [e0 as u32, e1 as u32, e2 as u32]
}

#[inline]
pub fn op1p_trysub1p(a: [u32; 3], b: [u32; 3]) -> [u32; 3] {
    let c0 = (a[0] as i64) - (b[0] as i64);
    let c1 = (a[1] as i64) - (b[1] as i64);
    let c2 = (a[2] as i64) - (b[2] as i64);

    let d0 = c0;
    let d1 = (d0 >> 32) + c1;
    let d2 = (d1 >> 32) + c2;

    if d2 < 0 {
        a
    } else {
        [d0 as u32, d1 as u32, d2 as u32]
    }
}

pub fn poly64(a: [u32; 3], b: [u32; 3], c: [u32; 3], x: &[u64]) -> u32 {
    if x.is_empty() {
        return b[0] & 0xfffff;
    }

    let mut h = [x[0] as u32, (x[0] >> 32) as u32, 0];
    for &v in &x[1..] {
        let v0 = v as u32;
        let v1 = (v >> 32) as u32;
        let d = op2p_digitsum_add1p(op3_mul3(c, h), [v0, v1, 0]);
        h = op1p_trysub1p(op1p_trysub1p(d, P64), P64);
    }

    let d = op2p_digitsum_add1p(op3_mul3(a, h), b);
    let e = op1p_trysub1p(op1p_trysub1p(d, P64), P64);

    (e[0] & 0xfffff) as u32
}
