//! Turns a palette of 256 entries of three bytes into 256 dwords: an entry's first byte shifted up
//! 16, its second shifted up 8, and its third; the top byte is 0. Entry 255 is done first.

use crate::errors::BadInput;
use crate::memory::session::PlainDword;

pub const ENTRIES: usize = 256;

/// The palette (256 entries of three bytes) turned into dwords, written to `to`.
pub(crate) fn palette_bytes_to_dwords(
    palette: &[u8; ENTRIES * 3],
    to: &[PlainDword; ENTRIES],
) -> Result<(), BadInput> {
    // From the last entry down. The palette is a separate buffer, so the order of the writes
    // changes nothing.
    for entry in (0..ENTRIES).rev() {
        let bytes = [
            palette[entry * 3],
            palette[entry * 3 + 1],
            palette[entry * 3 + 2],
        ];
        to[entry].set(rc2k_core::gfx::pixels::palette_entry(bytes));
    }
    Ok(())
}
