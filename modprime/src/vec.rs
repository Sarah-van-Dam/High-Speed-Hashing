// Assume w = 32, wbar = 64, l = 20, d = 64.

pub const L: usize = 20;
pub const D: usize = 64;

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
