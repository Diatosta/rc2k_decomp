//! The race's results file (`sub_41E6EC` / `sub_41E85A`, IDA 2026-10-17): its bytes packed
//! by runs (`race::pack_runs`) and a block sealed by a DES-like cipher of the game's
//! tables (`race::seal_block`, with `race::permute_bits`).

use crate::errors::BadInput;
use crate::race::owner::race;
use crate::system::owner::system;

/// `race::pack_runs` (0x41EFEC): ecx bytes at esi packed to edi: runs of 2 .. 0x80 equal
/// bytes as (length - 1, byte), other stretches of up to 0x80 as (-length, bytes); fewer
/// than three bytes, or one left, as one literal; then a dword 0 of which one byte
/// counts. Returns the bytes written (ecx). Every other register kept. Here: `source`
/// packed into the shared scratch (`g_pngInfo`, edi of its one caller); the packing (end
/// mark included) is returned with the count.
pub fn pack_runs(source: &[u8]) -> Result<(Vec<u8>, u32), BadInput> {
    // Converted: `rc2k_core::race::seal::pack_runs`.
    let mut packed = rc2k_core::race::seal::pack_runs(source);
    let len = packed.len() as u32;
    // The end mark is the first byte of a dword 0.
    packed.extend_from_slice(&[0, 0, 0]);
    system()
        .work
        .files
        .buffer
        .write_at(0, &packed, "the packed bytes")?;
    Ok((packed, len))
}

// The cipher's tables and variables (dword_4E052C .. as IDA shows them).

/// `race::seal_block` (0x402735): the block (`dword_4E0534` / `38`) enciphered with the key
/// (`dword_4E052C` / `30`) by the game's tables: the key's two halves (PC1), sixteen
/// subkeys (turns by `byte_4E01C7`, PC2) into `dword_4E04AC` ..; the block's initial
/// permutation, sixteen rounds (expansion, the subkey, eight S-boxes, the permutation,
/// the halves swapped), the final permutation into `value` (0x4E053C) and `dword_4E0540`.
/// Every register kept (`pusha` / `popa`).
pub fn seal_block() {
    // Converted: `rc2k_core::race::seal::seal_block`.
    {
        use crate::race::seal_tables::*;
        let cipher = rc2k_core::race::seal::Cipher {
            pc1: [&PC1_A, &PC1_B],
            pc2: &PC2,
            shifts: SHIFTS,
            initial: &INITIAL,
            expansion: &EXPANSION,
            permutation: &PERMUTATION,
            last: &LAST,
        };
        let w = &race().replay.seal_work;
        let sealed = rc2k_core::race::seal::seal_block(
            [w.key[0].get(), w.key[1].get()],
            [w.block[0].get(), w.block[1].get()],
            &cipher,
            |k, six| SBOXES[0x40 * k + six as usize],
        );
        for (cell, value) in w.subkeys.iter().zip(sealed.subkeys.iter().flatten()) {
            cell.set(*value);
        }
        w.sealed[0].set(sealed.block[0]);
        w.sealed[1].set(sealed.block[1]);
    }
}
