//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x300 bytes at `byte_4F4B30`: the mean of three components by their sum. One caller,
//! `start`, which calls it directly (2026-10-23). Who reads the table has not been looked up.

use crate::gfx::block::game_write;
use crate::gfx::modes::Slot as ModeSlot;

// ---- game data ----

const TABLE_RVA: ModeSlot = ModeSlot::at(0x4F4B30); // byte_4F4B30
const TABLE_LEN: usize = 0x300;

// ---- Rust version ----

pub unsafe fn fill_thirds_table() {
    // Converted: `rc2k_core::gfx::pixels::thirds_table`.
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    unsafe { game_write(TABLE_RVA, table) };
}
