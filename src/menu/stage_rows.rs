//! The event pages' row steps: the first row shown (`champ_view.first_row`) ten back or on, then
//! `menu::stage_range_caption`. `last` is `event.standings_count` - 9 (`menu::previous_stage_rows`,
//! `next_stage_rows`) or 0x5A - 9 (`previous_stage_rows_of_90`, `next_stage_rows_of_90`).
//! - back: at 10 or below (signed) the row is `last`, else ten less;
//! - on: at `last` or past (signed) the row is 1, else ten more.
//!
//! Left/right handlers of the event pages' items.

use crate::errors::Refused;
use crate::menu::owner::Menus;

/// `of_90` the fixed count, `on` the direction.
pub fn step_rows(menus: &Menus, of_90: bool, on: bool) -> Result<(), Refused> {
    let count = if of_90 {
        0x5A
    } else {
        menus.screens.event.standings_count.get()
    };
    let _last = count.wrapping_sub(9);
    let row: u32 = menus.play.champ_view.first_row.get();
    let row = rc2k_core::menu::steps::rows(row, count, on);
    menus.play.champ_view.first_row.set(row);
    crate::menu::stage_range_caption::stage_range_caption(&menus.play)?;
    Ok(())
}
