//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x300 bytes at `byte_4F4B30`: the mean of three components by their sum. One caller,
//! `start`, which calls it directly (2026-10-23). Who reads the table has not been looked up.

use crate::session::Bytes;

// ---- game data ----

/// IDA's address of the table.
pub const START: usize = 0x4F4B30;
const TABLE_LEN: usize = 0x300;

/// `byte_4F4B30`: n / 3 for the sums of three components (zero until filled).
pub static THIRDS: Bytes<TABLE_LEN> = Bytes::new([0; TABLE_LEN]);

/// Where the table is (`moved::MOVED`'s home for IDA 0x4F4B30).
pub fn home() -> usize {
    THIRDS.addr()
}

// ---- Rust version ----

pub fn fill_thirds_table() {
    // Converted: `rc2k_core::gfx::pixels::thirds_table`.
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    THIRDS.write(&table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&THIRDS, START, START + TABLE_LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), THIRDS.addr());
    }
}
