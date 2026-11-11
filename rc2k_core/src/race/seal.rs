//! The race's results file: its bytes packed by runs, the stage's time sealed, and a
//! block enciphered by the game's DES-like cipher (Converted from
//! `race::pack_runs` 0x41EFEC, `seal_stage_time` 0x41F29D, `permute_bits` 0x402848 and
//! `seal_block` 0x402735).

/// The longest time `seal_stage_time` seals (0x22550FF).
pub const MOST_TIME: u32 = 0x0225_50FF;

/// `race::seal_stage_time`: the first block's time (`time`, held to `MOST_TIME`
/// unsigned; `absent`, its +0xB0, in its place when that is negative) `xor` the key and
/// turned right by the key's low five bits. `time` is asked only when used.
pub fn seal_time(absent: u32, time: impl FnOnce() -> u32, key: u32) -> u32 {
    let v = if (absent as i32) >= 0 {
        time().min(MOST_TIME)
    } else {
        absent
    };
    (v ^ key).rotate_right(key & 0x1F)
}

/// `race::permute_bits`: the bits picked by `picks` (a table's count byte n is followed
/// by n + 1 picks: bit 5 set takes bit (pick & 0x1F) of the first input word, else of the second), gathered
/// from the top down: the first picks in the high half, the rest in the low.
pub fn permute_bits(picks: &[u8], eax: u32, edx: u32) -> u64 {
    picks.iter().fold(0u64, |bits, &b| {
        let from = if b & 0x20 != 0 { eax } else { edx };
        (bits << 1) | ((from >> (b & 0x1F)) & 1) as u64
    })
}

/// The halves of a `permute_bits` result: (high, low).
pub fn halves(bits: u64) -> (u32, u32) {
    ((bits >> 32) as u32, bits as u32)
}

/// The cipher's tables, as RAL.EXE keeps them (each permutation its picks, without the
/// count byte).
pub struct Cipher<'a> {
    /// The key's two halves (PC1: 0x4E015C, 0x4E0179).
    pub pc1: [&'a [u8]; 2],
    /// The subkeys' choice (PC2, 0x4E0196).
    pub pc2: &'a [u8],
    /// Whether each round turns the halves twice (`byte_4E01C7`).
    pub shifts: [u8; 16],
    /// The initial permutation (0x4E01D7).
    pub initial: &'a [u8],
    /// The expansion (0x4E0259).
    pub expansion: &'a [u8],
    /// The permutation after the S-boxes (0x4E028A).
    pub permutation: &'a [u8],
    /// The final permutation (0x4E0218).
    pub last: &'a [u8],
}

/// A 28-bit half turned left by one (`btr` bit 27, `adc` itself).
fn turn28(x: u32) -> u32 {
    let top = (x >> 27) & 1;
    ((x & !(1 << 27)) << 1) | top
}

/// What `seal_block` makes: the sixteen subkeys (high, low dwords, as the game keeps
/// them at 0x4E04AC) and the sealed block (`value`, `dword_4E0540`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sealed {
    pub subkeys: [[u32; 2]; 16],
    pub block: [u32; 2],
}

/// `race::seal_block`: `block` enciphered with `key`: the key's two halves (PC1), sixteen
/// subkeys (turned by `shifts`, PC2); the block's initial permutation, sixteen rounds
/// (expansion, the subkey, the eight S-boxes, the permutation, the halves swapped), the
/// final permutation. `sbox(k, six)` is S-box k's entry `six` (asked as the original reads
/// them).
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
        let mut eax = (e >> 32) as u32 ^ subkey[0];
        let mut edx = e as u32 ^ subkey[1];
        let mut ebx = 0u32;
        for k in 0..8 {
            eax = (eax << 6) | (edx >> 26);
            edx <<= 6;
            ebx <<= 4;
            ebx |= sbox(k, (eax >> 16) & 0x3F) as u32;
        }
        let f = permute_bits(t.permutation, ebx, ebx) as u32;
        (l, r) = (r, l ^ f);
    }
    let (a, b) = halves(permute_bits(t.last, r, l));
    Sealed {
        subkeys,
        block: [a, b],
    }
}

/// `race::pack_runs`: `from` packed: runs of 2 .. 0x80 equal bytes as (length - 1, the
/// byte), other stretches of up to 0x80 as (-length, the bytes); fewer than three bytes,
/// or one left, as one literal stretch; then the end mark 0. The bytes the original
/// counts (the end mark is the first byte of a dword 0 it writes).
pub fn pack_runs(from: &[u8]) -> Vec<u8> {
    let count = from.len() as u32;
    let src = |i: u32| from[i as usize];
    let mut out = Vec::with_capacity(from.len() + from.len() / 0x80 + 2);
    let mut s = 0u32;
    let mut left = count;
    let mut literal_all = count < 3;
    if !literal_all {
        loop {
            let al = src(s);
            if (left as i32) <= 1 {
                literal_all = true;
                break;
            }
            if al == src(s + 1) {
                let mut n = 2u32;
                let mut end = false;
                loop {
                    if n == left {
                        end = true;
                        break;
                    }
                    if al != src(s + n) {
                        break;
                    }
                    n += 1;
                    if n >= 0x80 {
                        break;
                    }
                }
                s += n;
                out.push((n as u8).wrapping_sub(1));
                out.push(al);
                if end {
                    break;
                }
                left -= n;
            } else {
                let mut n = 1u32;
                let mut al = al;
                let mut end = false;
                loop {
                    if n == left {
                        end = true;
                        break;
                    }
                    if al == src(s + n) {
                        n -= 1;
                        break;
                    }
                    al = src(s + n);
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
