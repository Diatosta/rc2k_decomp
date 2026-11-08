//! The event pages' row steps (agent E, 2026-10-19): the first row shown
//! (`g_champFirstRow`, 0x7556B8) ten back or on, then `menu::stage_range_caption`. `last` is `dword_75597C` - 9 (`menu::previous_stage_rows` 0x4A3736,
//! `next_stage_rows` 0x4A375F) or 0x5A - 9 (`previous_stage_rows_of_90` 0x4A378C,
//! `next_stage_rows_of_90` 0x4A37B5).
//! - back: at 10 or below (signed) the row is `last` (eax it), else ten less;
//! - on: eax `last`; at `last` or past (signed) the row is 1, else ten more.
//!
//! Left/right handlers of the event pages' items (menu tables 0x74FAFC ..).

use crate::errors::Refused;
use crate::menu::owner::Menus;

// g_captionText

/// The Rust version: `of_90` the fixed count, `on` the direction.
pub fn step_rows(menus: &Menus, of_90: bool, on: bool) -> Result<(), Refused> {
    let count = if of_90 {
        0x5A
    } else {
        menus.screens.event.count_a.get()
    };
    let _last = count.wrapping_sub(9);
    let row: u32 = menus.play.champ_view.first_row.get();
    // Converted: `rc2k_core::menu::steps::rows`.
    let row = rc2k_core::menu::steps::rows(row, count, on);
    menus.play.champ_view.first_row.set(row);
    crate::menu::stage_range_caption::stage_range_caption(&menus.play)?;
    Ok(())
}
