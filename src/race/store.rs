//! The race core's first values: the image's bytes of IDA 0x5BEC48 .. 0x747AD8, from the
//! PNG reader's memory list to the menus' tables (`store_data.rs`, generated), which each
//! of the race's statics starts from (`first_values`), and the figures the debug pages read
//! there that no typed field holds (`block_dword`).
//!
//! The image's zeroed tail after it (IDA 0x960000 .. 0xA8C180) is `race::buffers_store`.

/// IDA's address of the block's first byte.
const START: usize = 0x5BEC48;

/// The ranges of the block whose bytes no static of its own holds (IDA start, end): the
/// debug pages read their figures there (`block_dword`).
#[cfg(debug_ui)]
pub const OWN: [(usize, usize); 23] = [
    (0x5BEC48, 0x5BEC78),
    (0x5C2714, 0x5C291C),
    (0x5C5E50, 0x5C5F48),
    (0x5C5FB8, 0x5C6604),
    (0x5C6610, 0x5C6924),
    (0x5C6948, 0x5C694C),
    (0x5C7908, 0x5C790C),
    (0x5EA8C8, 0x5EB100),
    (0x5EB104, 0x60101C),
    (0x60230C, 0x60410C),
    (0x60526C, 0x60626C),
    (0x60B0BD, 0x60B2CC),
    (0x60B2D0, 0x613828),
    (0x61FE18, 0x620458),
    (0x620474, 0x621480),
    (0x6218B0, 0x6218F0),
    (0x6218F4, 0x621900),
    (0x621A84, 0x62290C),
    (0x622910, 0x6235E8),
    (0x628A44, 0x62DA44),
    (0x64098C, 0x64158C),
    (0x64BB88, 0x6D13FE),
    (0x71CA60, 0x731B82),
];

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

/// The dword at IDA's `ida` among the block's first values (the debug pages' figures that no
/// typed field holds: nothing writes them); none outside `OWN` or when it runs past the end of
/// its range.
#[cfg(debug_ui)]
pub fn block_dword(ida: usize) -> Option<u32> {
    let &(_, end) = OWN
        .iter()
        .find(|&&(start, end)| (start..end).contains(&ida))?;
    if ida + 4 > end {
        return None;
    }
    Some(u32::from_le_bytes(first_values::<4>(ida)))
}
