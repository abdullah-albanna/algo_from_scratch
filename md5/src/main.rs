use std::{io::Read as _, ops::RangeInclusive};

const A: u32 = 0x67452301u32;
const B: u32 = 0xefcdab89u32;
const C: u32 = 0x98badcfeu32;
const D: u32 = 0x10325476u32;

macro_rules! round {
    ( $a:ident, $b:ident, $c:ident, $d:ident, $k:expr, $i:expr, $shift:expr, $func:expr, $chunks:expr, $table:expr) => {
        *$a[0] = $b.wrapping_add(
            ($a[0]
                .wrapping_add($func(**$b, **$c, **$d))
                .wrapping_add($chunks[$k])
                .wrapping_add($table[$i]))
            .rotate_left(*$shift),
        )
    };
}

fn bit_pad(v: &mut Vec<u8>) {
    let bit_len = (v.len() as u64) * 8;
    v.push(0x80);

    // it's the same as (v.len() * 8) % 512 != 448
    while v.len() % 64 != 56 {
        v.push(0);
    }

    v.extend(bit_len.to_le_bytes());
}

fn create_table() -> Vec<u32> {
    (0..64)
        .map(|i| (2.0_f64.powf(32.0) * (f64::sin((i + 1) as f64).abs())) as u32)
        .collect()
}

fn f(x: u32, y: u32, z: u32) -> u32 {
    x & y | !x & z
}

fn g(x: u32, y: u32, z: u32) -> u32 {
    x & z | y & !z
}

fn h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

fn round(
    mut registers: [&mut u32; 4],
    table: &[u32],
    chunks: &[u32],
    ks: [usize; 16],
    shifts: [u32; 4],
    range: RangeInclusive<usize>,
    func: impl Fn(u32, u32, u32) -> u32,
) {
    for ((i, k), shift) in range.zip(ks).zip(shifts.iter().cycle()) {
        let (a, rest) = registers.split_at_mut(1);
        let [b, c, d] = rest else { unreachable!() };

        round!(a, b, c, d, k, i, shift, func, chunks, table);
        registers.rotate_right(1);
    }
}

fn round_1(a: &mut u32, b: &mut u32, c: &mut u32, d: &mut u32, table: &[u32], chunks: &[u32]) {
    let registers = [a, b, c, d];
    let shifts = [7, 12, 17, 22];
    let ks = core::array::from_fn(|i| i);

    round(registers, table, chunks, ks, shifts, 0..=15, f);
}

fn round_2(a: &mut u32, b: &mut u32, c: &mut u32, d: &mut u32, table: &[u32], chunks: &[u32]) {
    let registers = [a, b, c, d];
    let shifts = [5, 9, 14, 20];
    let ks = [1, 6, 11, 0, 5, 10, 15, 4, 9, 14, 3, 8, 13, 2, 7, 12];

    round(registers, table, chunks, ks, shifts, 16..=31, g);
}

fn round_3(a: &mut u32, b: &mut u32, c: &mut u32, d: &mut u32, table: &[u32], chunks: &[u32]) {
    let registers = [a, b, c, d];
    let shifts = [4, 11, 16, 23];
    let ks = [5, 8, 11, 14, 1, 4, 7, 10, 13, 0, 3, 6, 9, 12, 15, 2];

    round(registers, table, chunks, ks, shifts, 32..=47, h);
}

fn round_4(a: &mut u32, b: &mut u32, c: &mut u32, d: &mut u32, table: &[u32], chunks: &[u32]) {
    let registers = [a, b, c, d];
    let shifts = [6, 10, 15, 21];
    let ks = [0, 7, 14, 5, 12, 3, 10, 1, 8, 15, 6, 13, 4, 11, 2, 9];

    round(registers, table, chunks, ks, shifts, 48..=63, i);
}

fn create_md5_digest(mut v: Vec<u8>) -> String {
    let table = create_table();
    let mut a = A;
    let mut b = B;
    let mut c = C;
    let mut d = D;

    for chunk in v.chunks_exact_mut(64) {
        let chunk_32 = chunk
            .chunks(4)
            .map(|c| {
                let x: [u8; 4] = c.try_into().unwrap();
                u32::from_ne_bytes(x)
            })
            .collect::<Vec<_>>();

        round_1(&mut a, &mut b, &mut c, &mut d, &table, &chunk_32);
        round_2(&mut a, &mut b, &mut c, &mut d, &table, &chunk_32);
        round_3(&mut a, &mut b, &mut c, &mut d, &table, &chunk_32);
        round_4(&mut a, &mut b, &mut c, &mut d, &table, &chunk_32);

        a = A.wrapping_add(a);
        b = B.wrapping_add(b);
        c = C.wrapping_add(c);
        d = D.wrapping_add(d);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}",
        a.swap_bytes(),
        b.swap_bytes(),
        c.swap_bytes(),
        d.swap_bytes()
    )
}

fn main() {
    let mut message = String::default();
    std::io::stdin().read_to_string(&mut message).unwrap();

    let bytes = message.as_bytes();
    let mut padded_message = bytes.to_vec();

    bit_pad(&mut padded_message);
    println!("{}", create_md5_digest(padded_message));
}
