//! `service::toggle_repair_choice` (0x46B7C0): the item action of the service area's repair
//! items: the status (`dword_75AAA4`) and edit mode 0, the highlighted item's flag flipped,
//! the chosen repairs' time added up into `dword_75AC8C`, and by `budget - total` (signed):
//!   at least 900:  every mark (`dword_759BC2`, from entry 0 to the end mark) 0;
//!   0 .. 899:      the item's mark 1;
//!   below 0:       the flag flipped back and the status 1.
//! Then the total is added up and stored again.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Refuses (before anything) a highlighted item past the flag table and what
//! `chosen_repairs_time` refuses (an item without a time chosen: the original reads
//! through null), and a mark table without its end.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::service::state::flagged_item;
use crate::service::state::{flags, marks, store_flags, store_marks, time_of};

/// The Rust version (see the header).
pub fn toggle_repair_choice(menus: &Menus) -> Result<(), Refused> {
    let (screens, tables) = (&menus.screens, &menus.tables);
    // Converted: `rc2k_core::service::repairs::toggle_choice` (nothing written when it
    // refuses).
    let item = flagged_item(screens)?;
    let (mut f, mut m) = (flags(tables), marks(tables));
    let budget = screens.event.service_minutes.get() as i32;
    let (total, status) =
        rc2k_core::service::repairs::toggle_choice(&mut f, &mut m, item as usize, budget, |item| {
            time_of(tables, item)
        })
        .map_err(|e| BadInput(e.0))?;
    store_flags(tables, &f);
    store_marks(tables, &m);
    {
        screens.state.service_ui.status.set(status);
        screens.state.service_ui.box_shown.set(0u32);
        screens.state.driver_stats.repair_time.set(total as u32);
    }
    Ok(())
}
