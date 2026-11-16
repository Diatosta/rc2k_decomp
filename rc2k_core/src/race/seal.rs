//! The race's results file: bytes packed by runs, the stage's time sealed, and a block
//! enciphered by a DES-like cipher.

/// The longest time sealed (0x22550FF).
pub const MOST_TIME: u32 = 0x0225_50FF;

/// The first block's time (held to `MOST_TIME`, or `absent` when that is negative) xor the
/// key, rotated right by the key's low five bits. `time` is asked only when used.
pub fn seal_time(absent: u32, time: impl FnOnce() -> u32, key: u32) -> u32 {
    let v = if (absent as i32) >= 0 {
        time().min(MOST_TIME)
    } else {
        absent
    };
    (v ^ key).rotate_right(key & 0x1F)
}

/// Gathers bits by `picks` from the top down (count byte n is followed by n + 1 picks; bit 5
/// set takes bit (pick & 0x1F) of `first`, else of `second`).
pub fn permute_bits(picks: &[u8], first: u32, second: u32) -> u64 {
    picks.iter().fold(0u64, |bits, &b| {
        let from = if b & 0x20 != 0 { first } else { second };
        (bits << 1) | ((from >> (b & 0x1F)) & 1) as u64
    })
}

/// The halves of a `permute_bits` result: (high, low).
pub fn halves(bits: u64) -> (u32, u32) {
    ((bits >> 32) as u32, bits as u32)
}

/// The cipher's tables (each permutation its picks, without the count byte).
pub struct Cipher<'a> {
    /// The key's two halves (PC1).
    pub pc1: [&'a [u8]; 2],
    /// The subkeys' choice (PC2).
    pub pc2: &'a [u8],
    /// Whether each round turns the halves twice.
    pub shifts: [u8; 16],
    /// The initial permutation.
    pub initial: &'a [u8],
    /// The expansion.
    pub expansion: &'a [u8],
    /// The permutation after the S-boxes.
    pub permutation: &'a [u8],
    /// The final permutation.
    pub last: &'a [u8],
}

/// A 28-bit half rotated left by one.
fn turn28(x: u32) -> u32 {
    let top = (x >> 27) & 1;
    ((x & !(1 << 27)) << 1) | top
}

/// What `seal_block` makes: the sixteen subkeys (high, low) and the sealed block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sealed {
    pub subkeys: [[u32; 2]; 16],
    pub block: [u32; 2],
}

/// `block` enciphered with `key`: PC1 halves, sixteen subkeys, initial permutation, sixteen
/// rounds, final permutation. `sbox(k, six)` is S-box k's entry `six`.
pub fn seal_block(
    key: [u32; 2],
    block: [u32; 2],
    t: &Cipher,
    sbox: impl Fn(usize, u32) -> u8,
) -> Sealed {
    let (_, mut c) = halves(permute_bits(t.pc1[0], key[0], key[1]));
    let (_, mut d) = halves(permute_bits(t.pc1[1], key[0], key[1]));
    let mut subkeys = [[0u32; 2]; 16];
    for (n, subkey) in subkeys.iter_mut().enumerate() {
        c = turn28(c);
        d = turn28(d);
        if t.shifts[n] != 0 {
            c = turn28(c);
            d = turn28(d);
        }
        let (hi, lo) = halves(permute_bits(t.pc2, c, d));
        *subkey = [hi, lo];
    }
    let (mut l, mut r) = halves(permute_bits(t.initial, block[0], block[1]));
    for subkey in &subkeys {
        let e = permute_bits(t.expansion, r, r);
        let mut high = (e >> 32) as u32 ^ subkey[0];
        let mut low = e as u32 ^ subkey[1];
        let mut mixed = 0u32;
        for k in 0..8 {
            high = (high << 6) | (low >> 26);
            low <<= 6;
            mixed <<= 4;
            mixed |= sbox(k, (high >> 16) & 0x3F) as u32;
        }
        let f = permute_bits(t.permutation, mixed, mixed) as u32;
        (l, r) = (r, l ^ f);
    }
    let (a, b) = halves(permute_bits(t.last, r, l));
    Sealed {
        subkeys,
        block: [a, b],
    }
}

/// `from` packed: runs of 2 ..= 0x80 equal bytes as (length - 1, byte), other stretches of up
/// to 0x80 as (-length, bytes); fewer than three bytes, or one left, as one literal
/// stretch; then the end mark 0. Returns the byte count.
pub fn pack_runs(from: &[u8]) -> Vec<u8> {
    let count = from.len() as u32;
    let src = |i: u32| from[i as usize];
    let mut out = Vec::with_capacity(from.len() + from.len() / 0x80 + 2);
    let mut s = 0u32;
    let mut left = count;
    let mut literal_all = count < 3;
    if !literal_all {
        loop {
            let byte = src(s);
            if (left as i32) <= 1 {
                literal_all = true;
                break;
            }
            if byte == src(s + 1) {
                let mut n = 2u32;
                let mut end = false;
                loop {
                    if n == left {
                        end = true;
                        break;
                    }
                    if byte != src(s + n) {
                        break;
                    }
                    n += 1;
                    if n >= 0x80 {
                        break;
                    }
                }
                s += n;
                out.push((n as u8).wrapping_sub(1));
                out.push(byte);
                if end {
                    break;
                }
                left -= n;
            } else {
                let mut n = 1u32;
                let mut previous = byte;
                let mut end = false;
                loop {
                    if n == left {
                        end = true;
                        break;
                    }
                    if previous == src(s + n) {
                        n -= 1;
                        break;
                    }
                    previous = src(s + n);
                    n += 1;
                    if n >= 0x81 {
                        n -= 1;
                        break;
                    }
                }
                if !end {
                    left -= n;
                }
                out.push((n as u8).wrapping_neg());
                out.extend_from_slice(&from[s as usize..(s + n) as usize]);
                s += n;
                if end {
                    break;
                }
            }
        }
    }
    if literal_all {
        out.push((left as u8).wrapping_neg());
        out.extend_from_slice(&from[s as usize..(s + left) as usize]);
    }
    out.push(0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time() {
        assert_eq!(seal_time(0, || 5, 0), 5);
        assert_eq!(seal_time(0, || u32::MAX, 0), MOST_TIME);
        assert_eq!(seal_time(u32::MAX, || unreachable!(), 0), u32::MAX);
        assert_eq!(seal_time(0, || 1, 0x21), (1u32 ^ 0x21).rotate_right(1));
    }

    #[test]
    fn permutes() {
        // The first word's bit 0, the second's bit 31, the first's bit 31.
        assert_eq!(permute_bits(&[0x20, 0x1F, 0x3F], 1, 0x8000_0000), 0b110);
        assert_eq!(halves(0x1_0000_0002), (1, 2));
        assert_eq!(turn28(0x0800_0001), 3);
    }

    #[test]
    fn packs() {
        assert_eq!(pack_runs(&[]), [0, 0]);
        assert_eq!(pack_runs(&[7, 7]), [0xFE, 7, 7, 0]);
        assert_eq!(pack_runs(&[5, 5, 5, 5]), [3, 5, 0]);
        assert_eq!(pack_runs(&[1, 2, 3, 3, 3]), [0xFE, 1, 2, 2, 3, 0]);
        assert_eq!(pack_runs(&[1, 2, 3]), [0xFD, 1, 2, 3, 0]);
        assert_eq!(pack_runs(&[4, 4, 4, 9]), [2, 4, 0xFF, 9, 0]);
        let long = [8u8; 0x90];
        assert_eq!(pack_runs(&long), [0x7F, 8, 0x0F, 8, 0]);
    }
}
