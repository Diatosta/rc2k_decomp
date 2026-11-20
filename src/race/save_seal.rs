//! The race's results file (`race::open_replay` / `race::close_replay`): its bytes packed by runs
//! (`race::pack_runs`) and a block sealed by a DES-like cipher (`race::seal_block`, with
//! `race::permute_bits`).

use crate::race::seal_store::SealWork;

/// Packs `source` by runs: runs of 2 .. 0x80 equal bytes as (length - 1, byte), other stretches
/// of up to 0x80 as (-length, bytes); fewer than three bytes, or one left, as one literal; then
/// a dword 0 of which one byte counts. Returns the packing (end mark included) with the count of
/// bytes written.
pub fn pack_runs(source: &[u8]) -> (Vec<u8>, u32) {
    // Logic in `rc2k_core::race::seal::pack_runs`.
    let mut packed = rc2k_core::race::seal::pack_runs(source);
    let len = packed.len() as u32;
    // The end mark is the first byte of a dword 0.
    packed.extend_from_slice(&[0, 0, 0]);
    (packed, len)
}

/// Enciphers the block (`SealWork`'s block) with its key: the key's two halves (PC1), sixteen
/// subkeys (turns by, PC2) into the subkeys; the block's initial permutation, sixteen rounds
/// (expansion, the subkey, eight S-boxes, the permutation, the halves swapped), the final
/// permutation into the value.
pub fn seal_block(w: &mut SealWork) {
    // Logic in `rc2k_core::race::seal::seal_block`.
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
        let sealed = rc2k_core::race::seal::seal_block(w.key, w.block, &cipher, |k, six| {
            SBOXES[0x40 * k + six as usize]
        });
        for (slot, value) in w.subkeys.iter_mut().zip(sealed.subkeys.iter().flatten()) {
            *slot = *value;
        }
        w.sealed = sealed.block;
    }
}
