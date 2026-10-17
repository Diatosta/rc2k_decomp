//! The race's results file (`sub_41E6EC` / `sub_41E85A`, IDA 2026-10-17): its bytes packed
//! by runs (`race::pack_runs`) and a block sealed by a DES-like cipher of the game's
//! tables (`race::seal_block`, with `race::permute_bits`).

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::race_shell::permute_bits;
use crate::service::harness::{Region, at};

/// `race::pack_runs` (0x41EFEC): ecx bytes at esi packed to edi: runs of 2 .. 0x80 equal
/// bytes as (length - 1, byte), other stretches of up to 0x80 as (-length, bytes); fewer
/// than three bytes, or one left, as one literal; then a dword 0 of which one byte
/// counts. Returns the bytes written (ecx). Every other register kept.
pub unsafe fn pack_runs(from: u32, count: u32, to: u32) -> u32 {
    unsafe {
        let src = |i: u32| -> u8 { *((from.wrapping_add(i)) as *const u8) };
        let mut s = 0u32; // esi - from
        let mut d = to; // edi
        let mut left = count; // edx
        let put = |d: &mut u32, b: u8| {
            *(*d as *mut u8) = b;
            *d += 1;
        };
        let copy = |d: &mut u32, s: &mut u32, n: u32| {
            for _ in 0..n {
                *(*d as *mut u8) = src(*s);
                *d += 1;
                *s += 1;
            }
        };
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
                    put(&mut d, (n as u8).wrapping_sub(1));
                    put(&mut d, al);
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
                    put(&mut d, (n as u8).wrapping_neg());
                    copy(&mut d, &mut s, n);
                    if end {
                        break;
                    }
                }
            }
        }
        if literal_all {
            put(&mut d, (left as u8).wrapping_neg());
            copy(&mut d, &mut s, left);
        }
        (d as *mut u32).write_unaligned(0);
        d + 1 - to
    }
}

// The cipher's tables and variables (dword_4E052C .. as IDA shows them).
const KEY_RVA: usize = rva(0x4E052C); // the key (two dwords)
const BLOCK_RVA: usize = rva(0x4E0534); // the block (two dwords)
const SUBKEYS_RVA: usize = rva(0x4E04AC); // sixteen subkeys (two dwords each)
pub const SEALED_RVA: [usize; 2] = [rva(0x4E053C), rva(0x4E0540)]; // `value`, dword_4E0540
const PC1_A: usize = rva(0x4E015C);
const PC1_B: usize = rva(0x4E0179);
const PC2: usize = rva(0x4E0196);
const SHIFTS: usize = rva(0x4E01C7); // byte_4E01C7: 16, a second turn when not 0
const IP: usize = rva(0x4E01D7);
const EXPANSION: usize = rva(0x4E0259);
const SBOXES: usize = rva(0x4E02AB); // 8 x 0x40
const PERMUTATION: usize = rva(0x4E028A);
const FP: usize = rva(0x4E0218);

/// `permute_bits` of edx:eax by the table at `table`: (esi, edi).
unsafe fn permute(eax: u32, edx: u32, table: usize) -> (u32, u32) {
    let g = unsafe {
        permute_bits(&Regs {
            eax,
            edx,
            ebx: game_addr(table) as u32,
            ..Regs::default()
        })
    };
    (g.esi, g.edi)
}

/// Bits of `from` picked by a count-and-picks table (`bt from, pick`), gathered from the
/// top (`adc` low, `adc` high): the expansion's 48 into two dwords, the permutation's 32.
unsafe fn pick(from: u32, table: usize) -> u64 {
    unsafe {
        let mut at = game_addr(table) as *const u8;
        let count = *at as u32;
        at = at.add(1);
        let mut bits = 0u64;
        for _ in 0..=count {
            let b = *at as u32;
            at = at.add(1);
            bits = (bits << 1) | ((from >> (b & 0x1F)) & 1) as u64;
        }
        bits
    }
}

/// A 28-bit half turned left by one (`btr` bit 27, `adc` itself).
fn turn28(x: u32) -> u32 {
    let top = (x >> 27) & 1;
    ((x & !(1 << 27)) << 1) | top
}

/// `race::seal_block` (0x402735): the block (`dword_4E0534` / `38`) enciphered with the key
/// (`dword_4E052C` / `30`) by the game's tables: the key's two halves (PC1), sixteen
/// subkeys (turns by `byte_4E01C7`, PC2) into `dword_4E04AC` ..; the block's initial
/// permutation, sixteen rounds (expansion, the subkey, eight S-boxes, the permutation,
/// the halves swapped), the final permutation into `value` (0x4E053C) and `dword_4E0540`.
/// Every register kept (`pusha` / `popa`).
pub unsafe fn seal_block() {
    unsafe {
        let key: [u32; 2] = game_read(KEY_RVA);
        let (_, c0) = permute(key[0], key[1], PC1_A);
        let (_, d0) = permute(key[0], key[1], PC1_B);
        let (mut c, mut d) = (c0, d0);
        for n in 0..16usize {
            c = turn28(c);
            d = turn28(d);
            if game_read::<u8>(SHIFTS + n) != 0 {
                c = turn28(c);
                d = turn28(d);
            }
            let (hi, lo) = permute(c, d, PC2);
            game_write(SUBKEYS_RVA + 8 * n, hi);
            game_write(SUBKEYS_RVA + 8 * n + 4, lo);
        }
        let block: [u32; 2] = game_read(BLOCK_RVA);
        let (mut l, mut r) = permute(block[0], block[1], IP);
        for n in 0..16usize {
            let e = pick(r, EXPANSION);
            let mut eax = (e >> 32) as u32 ^ game_read::<u32>(SUBKEYS_RVA + 8 * n);
            let mut edx = e as u32 ^ game_read::<u32>(SUBKEYS_RVA + 8 * n + 4);
            let mut ebx = 0u32;
            for k in 0..8usize {
                eax = (eax << 6) | (edx >> 26);
                edx <<= 6;
                ebx <<= 4;
                let six = (eax >> 16) & 0x3F;
                ebx |= game_read::<u8>(SBOXES + 0x40 * k + six as usize) as u32;
            }
            let f = pick(ebx, PERMUTATION) as u32;
            (l, r) = (r, l ^ f);
        }
        let (a, b) = permute(r, l, FP);
        game_write(SEALED_RVA[0], a);
        game_write(SEALED_RVA[1], b);
    }
}

/// What `seal_block` writes, and its inputs.
pub fn seal_regions() -> Vec<Region> {
    vec![
        (game_addr(SUBKEYS_RVA) as u32, 0x80),
        at(SEALED_RVA[0]),
        at(SEALED_RVA[1]),
    ]
}

pub fn seal_inputs() -> Vec<Region> {
    vec![
        (game_addr(KEY_RVA) as u32, 8),
        (game_addr(BLOCK_RVA) as u32, 8),
    ]
}
