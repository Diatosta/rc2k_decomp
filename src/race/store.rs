//! The race core's first values: the image's bytes of IDA 0x5BEC48 .. 0x747AD8, from the
//! PNG reader's memory list to the menus' tables (`store_data.rs`, generated), which each
//! of the race's statics starts from (`first_values`).
//!
//! The image's zeroed tail after it (IDA 0x960000 .. 0xA8C180) is `race::buffers_store`.

/// IDA's address of the block's first byte.
const START: usize = 0x5BEC48;

/// The image's first values of the `N` bytes from IDA's `start` in the block (a left
/// piece's static).
pub const fn first_values<const N: usize>(start: usize) -> [u8; N] {
    let pieces = crate::race::store_data::PIECES;
    let from = start - START;
    let mut bytes = [0u8; N];
    let mut p = 0;
    while p < pieces.len() {
        let piece_at = pieces[p].start - START;
        let runs = pieces[p].runs;
        let mut r = 0;
        while r < runs.len() {
            let (rel, run) = runs[r];
            let offset = piece_at + rel;
            // Only the overlap of the run with the wanted range (a long run was walked whole).
            let mut k = from.saturating_sub(offset);
            while k < run.len() && offset + k < from + N {
                bytes[offset + k - from] = run[k];
                k += 1;
            }
            r += 1;
        }
        p += 1;
    }
    bytes
}
