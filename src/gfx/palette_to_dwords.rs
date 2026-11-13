//! `gfx::palette_to_dwords` (0x405DDE): turns a palette of 256 entries of three bytes into
//! 256 dwords.
//!
//! Takes: the three-byte palette, the dwords
//! A dword is the entry's first byte shifted up 16, its second shifted up 8, and its third;
//! the top byte is 0. Entry 255 is done first. Four callers, the picture readers.
//! A palette or dwords field too short is refused. The name is a reading.

use crate::cells::Store;
use crate::errors::BadInput;

pub const ENTRIES: usize = 256;

/// The palette is read at byte `from_at` of `from`; the dwords go to the start of `to`.
pub fn palette_to_dwords(from: &dyn Store, from_at: usize, to: &dyn Store) -> Result<(), BadInput> {
    let mut palette = [0u8; ENTRIES * 3];
    from.load(from_at, &mut palette)?;
    palette_bytes_to_dwords(&palette, to)
}

/// The same for a palette held as bytes.
pub fn palette_bytes_to_dwords(
    palette: &[u8; ENTRIES * 3],
    to: &dyn Store,
) -> Result<(), BadInput> {
    // From the last entry down, as the original goes: it matters if the two overlap. The
    // bytes are read first (an entry's three bytes lie below its dword, so the order of the
    // writes cannot change a read that comes after it).
    let mut dwords = Vec::with_capacity(ENTRIES * 4);
    for entry in 0..ENTRIES {
        // Converted: `rc2k_core::gfx::pixels::palette_entry`.
        let bytes = [
            palette[entry * 3],
            palette[entry * 3 + 1],
            palette[entry * 3 + 2],
        ];
        dwords.extend_from_slice(&rc2k_core::gfx::pixels::palette_entry(bytes).to_le_bytes());
    }
    to.store(0, &dwords)
}
