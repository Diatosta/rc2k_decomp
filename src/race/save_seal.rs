//! The race's results file (`sub_41E6EC` / `sub_41E85A`, IDA 2026-10-17): its bytes packed
//! by runs (`race::pack_runs`) and a block sealed by a DES-like cipher of the game's
//! tables (`race::seal_block`, with `race::permute_bits`).

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at};

/// `race::pack_runs` (0x41EFEC): ecx bytes at esi packed to edi: runs of 2 .. 0x80 equal
/// bytes as (length - 1, byte), other stretches of up to 0x80 as (-length, bytes); fewer
/// than three bytes, or one left, as one literal; then a dword 0 of which one byte
/// counts. Returns the bytes written (ecx). Every other register kept.
pub unsafe fn pack_runs(from: u32, count: u32, to: u32) -> u32 {
    // Converted: `rc2k_core::race::seal::pack_runs` (the source read whole first: the
    // callers' buffers do not overlap).
    unsafe {
        let source = std::slice::from_raw_parts(from as *const u8, count as usize);
        let packed = rc2k_core::race::seal::pack_runs(source);
        std::ptr::copy_nonoverlapping(packed.as_ptr(), to as *mut u8, packed.len());
        // The end mark is the first byte of a dword 0.
        std::ptr::write_bytes((to as *mut u8).add(packed.len()), 0, 3);
        packed.len() as u32
    }
}

// The cipher's tables and variables (dword_4E052C .. as IDA shows them).
const KEY_RVA: usize = rva(0x4E052C); // the key (two dwords)
pub const BLOCK_RVA: usize = rva(0x4E0534); // the block (two dwords)
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

/// A count-and-picks table's picks (the count byte n, then n + 1 picks), read whole as
/// the original reads them.
pub unsafe fn picks<'a>(at: u32) -> &'a [u8] {
    unsafe {
        let count = *(at as *const u8) as usize;
        std::slice::from_raw_parts((at + 1) as *const u8, count + 1)
    }
}

/// `race::seal_block` (0x402735): the block (`dword_4E0534` / `38`) enciphered with the key
/// (`dword_4E052C` / `30`) by the game's tables: the key's two halves (PC1), sixteen
/// subkeys (turns by `byte_4E01C7`, PC2) into `dword_4E04AC` ..; the block's initial
/// permutation, sixteen rounds (expansion, the subkey, eight S-boxes, the permutation,
/// the halves swapped), the final permutation into `value` (0x4E053C) and `dword_4E0540`.
/// Every register kept (`pusha` / `popa`).
pub unsafe fn seal_block() {
    // Converted: `rc2k_core::race::seal::seal_block`.
    unsafe {
        let table = |r: usize| picks(game_addr(r) as u32);
        let cipher = rc2k_core::race::seal::Cipher {
            pc1: [table(PC1_A), table(PC1_B)],
            pc2: table(PC2),
            shifts: game_read(SHIFTS),
            initial: table(IP),
            expansion: table(EXPANSION),
            permutation: table(PERMUTATION),
            last: table(FP),
        };
        let sealed = rc2k_core::race::seal::seal_block(
            game_read(KEY_RVA),
            game_read(BLOCK_RVA),
            &cipher,
            |k, six| game_read::<u8>(SBOXES + 0x40 * k + six as usize),
        );
        game_write(SUBKEYS_RVA, sealed.subkeys);
        game_write(SEALED_RVA[0], sealed.block[0]);
        game_write(SEALED_RVA[1], sealed.block[1]);
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
