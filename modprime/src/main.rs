// NOTE: All numbers are represented with the least significant parts first.

pub const PRIME: [u32; 3] = [0xffffffff, 0xffffffff, (1 << 25) - 1];

pub fn multiply_add(a: [u32; 3], b: [u32; 3], x: [u32; 2]) -> [u32; 5] {
    let a0x0 = (a[0] as u64) * (x[0] as u64);
    let a1x0 = (a[1] as u64) * (x[0] as u64);
    let a2x0 = (a[2] as u64) * (x[0] as u64);
    let a0x1 = (a[0] as u64) * (x[1] as u64);
    let a1x1 = (a[1] as u64) * (x[1] as u64);
    let a2x1 = (a[2] as u64) * (x[1] as u64);

    let c0 = a0x0;
    let c1 = (c0 >> 32) + a1x0;
    let c2 = (c1 >> 32) + a2x0;
    let c3 = c2 >> 32;

    let d0 = c0 & 0xffffffff;
    let d1 = (d0 >> 32) + (c1 & 0xffffffff) + a0x1;
    let d2 = (d1 >> 32) + (c2 & 0xffffffff) + a1x1;
    let d3 = (d2 >> 32) + (c3 & 0xffffffff) + a2x1;
    let d4 = d3 >> 32;

    // TODO: We could reduce ax mod p before adding b.

    let e0 = (d0 & 0xffffffff) + (b[0] as u64);
    let e1 = (e0 >> 32) + (d1 & 0xffffffff) + (b[1] as u64);
    let e2 = (e1 >> 32) + (d2 & 0xffffffff) + (b[2] as u64);
    let e3 = (e2 >> 32) + (d3 & 0xffffffff);
    let e4 = (e3 >> 32) + (d4 & 0xffffffff); // TODO: Not strictly necessary?

    [e0 as u32, e1 as u32, e2 as u32, e3 as u32, e4 as u32]
}

pub fn modulo(y: [u32; 5], m: u32) -> u32 {
    let c0 = y[0]; // 32 bits
    let c1 = y[1]; // 32 bits
    let c2 = y[2] & 0x1ffffff; // 25 bits

    let d0 = ((y[3] & 0x1ffffff) << 7) | (y[2] >> 25);
    let d1 = ((y[4] & 0x1ffffff) << 7) | (y[3] >> 25);
    let d2 = y[4] >> 25;

    let e0 = (c0 as u64) + (d0 as u64);
    let e1 = (e0 >> 32) + (c1 as u64) + (d1 as u64);
    let e2 = (e1 >> 32) + (c2 as u64) + (d2 as u64);
    let e3 = e2 >> 32;

    assert_eq!(0, e3);

    let mut e = [e0 as u32, e1 as u32, e2 as u32];

    loop {
        // Since e = a + b for a,b in [p+1], we risk e > p and even e = 2p.

        let f0 = (e[0] as i64) - 0xffffffff;
        let f1 = (f0 >> 32) + (e[1] as i64) - 0xffffffff;
        let f2 = (f1 >> 32) + (e[2] as i64) - 0x1ffffff;

        if f2 < 0 {
            break;
        }

        e = [f0 as u32, f1 as u32, f2 as u32];
    }

    //println!("{:?} = {:?} : {:?} equiv {:?} (mod 2^89 - 1)", y, [c0, c1, c2], [d0, d1, d2], e);

    // Russian peasant multiplication by 1

    let mut k = 1;
    let mut r = 0;

    while e != [0, 0, 0] {
        //println!("e: {:?}, k: {:?}, r: {:?}, m: {:?}", e, k, r, m);

        if e[0] & 1 == 1 {
            r += k;
            if r >= m as u64 {
                r -= m as u64;
            }
        }

        e[0] = (e[1] << 31) | (e[0] >> 1);
        e[1] = (e[2] << 31) | (e[1] >> 1);
        e[2] = e[2] >> 1;

        k *= 2;
        if k >= m as u64 {
            k -= m as u64;
        }
    }

    assert_eq!([0, 0, 0], e);
    assert!(r < m as u64, "{} < {}", r, m);

    r as u32
}

pub fn mod_prime(m: u32, a: [u32; 3], b: [u32; 3], x: [u32; 2]) -> u32 {
    let y = multiply_add(a, b, x);
    let r = modulo(y, m);
    r
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
    let r = modulo([273, 0, 0, 0, 0], 11);
    assert_eq!(9, r);
}

#[test]
fn test_modulo_big() {
    let r = modulo(
        [0x77777777, 0x11111111, 0xdddddddd, 0xbbbbbbbb, 0x22222222],
        0x44444444,
    );
    assert_eq!(603979885, r);
}

#[test]
fn test_modulo_max1() {
    let r = modulo(
        [0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff],
        0xffffffff,
    );
    assert_eq!(127, r);
}

#[test]
fn test_modulo_max2() {
    // p mod p = 0, so (p mod p) mod m = 0.
    let r = modulo([0xffffffff, 0xffffffff, 0x1ffffff, 0, 0], 0xffffffff);
    assert_eq!(0, r);
}

#[test]
fn test_modulo_max3() {
    // (p - 1 mod p) mod m
    let r = modulo([0xfffffffe, 0xffffffff, 0x1ffffff, 0, 0], 0xffffffff);
    assert_eq!(33554430, r);
}

// TODO: Test mod_prime.

fn main() {
    // do nothing for now
}
