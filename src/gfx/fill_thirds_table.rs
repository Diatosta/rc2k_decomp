//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! 0x300 bytes at `thirds`: the mean of three components by their sum. One caller,
//! `start`, which calls it directly. Who reads the table has not been looked up.
#[cfg(test)]
use crate::gfx::owner::gfx;

// ---- game data ----

pub const TABLE_LEN: usize = 0x300;

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
        crate::test_support::first_values::assert_block_starts_as_the_games(&gfx().drawing.thirds);
    }
}
