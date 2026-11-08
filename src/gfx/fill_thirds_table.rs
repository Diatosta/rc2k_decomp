//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! 0x300 bytes at `byte_4F4B30`: the mean of three components by their sum. One caller,
//! `start`, which calls it directly (2026-10-23). Who reads the table has not been looked up.
#[cfg(test)]
use crate::gfx::owner::gfx;

// ---- game data ----

/// IDA's address of the table.
pub const START: usize = 0x4F4B30;
pub const TABLE_LEN: usize = 0x300;

// ---- Rust version ----

pub fn fill_thirds_table(gfx: &crate::gfx::owner::Gfx) {
    // Converted: `rc2k_core::gfx::pixels::thirds_table`.
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    gfx.drawing.thirds.write(&table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&gfx().drawing.thirds, START, START + TABLE_LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(
            crate::moved::place(START),
            gfx().drawing.thirds.addr() as usize
        );
    }
}
