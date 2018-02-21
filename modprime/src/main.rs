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

    let e0 = (d0 & 0xffffffff) + (b[0] as u64);
    let e1 = (e0 >> 32) + (d1 & 0xffffffff) + (b[1] as u64);
    let e2 = (e1 >> 32) + (d2 & 0xffffffff) + (b[2] as u64);
    let e3 = (e2 >> 32) + (d3 & 0xffffffff);
    let e4 = (e3 >> 32) + (d4 & 0xffffffff); // TODO: Not strictly necessary?

    [e0 as u32, e1 as u32, e2 as u32, e3 as u32, e4 as u32]
}

#[test]
fn test_multiply_small() {
    let r = multiply_add([1, 0, 0], [0, 0, 0], [5, 0]);
    assert_eq!([5, 0, 0, 0, 0], r);
}

#[test]
fn test_multiply_big() {
    let r = multiply_add([0x77777777, 0xdddddddd, 0x22222222], [0, 0, 0], [0xeeeeeeee, 0x33333333]);
    assert_eq!([0xd4c3b2a2, 0xcccccccc, 0x56789abb, 0x789abcdf, 0x6d3a06d], r);
}

#[test]
fn test_multiply_max() {
    let r = multiply_add([0xffffffff, 0xffffffff, 0xffffffff], [0, 0, 0], [0xffffffff, 0xffffffff]);
    assert_eq!([1, 0, 4294967295, 4294967294, 4294967295], r);
}

#[test]
fn test_multiply_add_small() {
    let r = multiply_add([1, 0, 0], [3, 0, 0], [5, 0]);
    assert_eq!([8, 0, 0, 0, 0], r);
}

#[test]
fn test_multiply_add_big() {
    let r = multiply_add([0x77777777, 0xdddddddd, 0x22222222], [0x55555555, 0xcccccccc, 0xffffffff], [0xeeeeeeee, 0x33333333]);
    assert_eq!([706283511, 2576980377, 1450744507, 2023406816, 114532461], r);
}

#[test]
fn test_multiply_add_max() {
    let r = multiply_add([0xffffffff, 0xffffffff, 0xffffffff], [0xffffffff, 0xffffffff, 0xffffffff], [0xffffffff, 0xffffffff]);
    assert_eq!([0, 0, 0xffffffff, 0xffffffff, 0xffffffff], r);
}

fn main() {
    let a = [1, 0, 0];
    let b = [2, 0, 0];
    let x = [5, 0];

    let z = multiply_add(a, b, x);

    println!("{:?} * {:?} + {:?} = {:?}", a, x, b, z);
}
