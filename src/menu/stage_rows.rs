//! The event pages' row steps (agent E, 2026-10-19): the first row shown
//! (`g_champFirstRow`, 0x7556B8) ten back or on, then `menu::stage_range_caption`. Not
//! `pusha`'d. `last` is `dword_75597C` - 9 (`menu::previous_stage_rows` 0x4A3736,
//! `next_stage_rows` 0x4A375F) or 0x5A - 9 (`previous_stage_rows_of_90` 0x4A378C,
//! `next_stage_rows_of_90` 0x4A37B5).
//! - back: at 10 or below (signed) the row is `last` (eax it), else ten less;
//! - on: eax `last`; at `last` or past (signed) the row is 1, else ten more.
//!
//! Left/right handlers of the event pages' items (menu tables 0x74FAFC ..).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};

const FIRST_RVA: usize = rva(0x7556B8); // g_champFirstRow
const COUNT_RVA: usize = rva(0x75597C); // dword_75597C
// g_captionText

/// The Rust version: `of_90` the fixed count, `on` the direction.
pub fn step_rows(of_90: bool, on: bool) -> Result<(), Refused> {
    unsafe {
        let count = if of_90 {
            0x5A
        } else {
            game_read::<u32>(COUNT_RVA)
        };
        let _last = count.wrapping_sub(9);
        let row: u32 = game_read(FIRST_RVA);
        // Converted: `rc2k_core::menu::steps::rows`.
        let row = rc2k_core::menu::steps::rows(row, count, on);
        game_write(FIRST_RVA, row);
        crate::menu::stage_range_caption::stage_range_caption();
        Ok(())
    }
}
