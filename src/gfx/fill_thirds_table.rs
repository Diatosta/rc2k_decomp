//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! 0x300 bytes at `thirds`: the mean of three components by their sum. One caller,
//! `start`, which calls it directly. Who reads the table has not been looked up.

// ---- game data ----

pub const TABLE_LEN: usize = 0x300;

pub fn fill_thirds_table(gfx: &crate::gfx::owner::Gfx) {
    // Converted: `rc2k_core::gfx::pixels::thirds_table`.
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    gfx.drawing.thirds.write(&table);
}
