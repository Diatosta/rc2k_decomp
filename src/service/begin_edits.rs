//! The four item actions that open an edit page of the service area that costs service
//! time (`service::begin_gear_ratio_edit` .. `begin_suspension_stiffness_edit`, 0x46B674,
//! 0x46B6BD, 0x46B72E, 0x46B777): if the highlighted item is chosen already (its flag 1),
//! or the service time still covers the chosen repairs plus this item's time
//! (`budget - (total + time)`, signed, at least 0), the value at the edit's start is the
//! chosen one, edit mode flips (`xor 1`) and the edit page is the item's. Else nothing.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Refuses (before anything) a highlighted item past the flag table (the original reads
//! past it). Self-test: the original and the Rust version on the variables, each flag,
//! budgets around the boundary, edit mode 0 and 1, put back.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::service::state::{flagged_item, repair_flag_in};
use crate::session::Dword;

/// One page's own variables.
#[derive(Clone, Copy)]
pub struct Edit {
    /// The page number written to `dword_75AAA0`.
    pub page: u32,
    /// The item's service time (`dword_75AB90` ..).
    pub time: &'static Dword,
    /// The chosen value, copied ...
    pub chosen: &'static Dword,
    /// ... to the value at the edit's start.
    pub start: &'static Dword,
}

/// The Rust version (see the header).
pub fn begin_edit(e: Edit) -> Result<(), Refused> {
    let menus = menus();
    let screens = &menus.screens;
    let item = flagged_item(screens)?;

    // Converted: `rc2k_core::service::repairs::may_edit`.
    if rc2k_core::service::repairs::may_edit(
        repair_flag_in(&menus.tables, item),
        screens.state.driver_stats.repair_time.get() as i32,
        e.time.get() as i32,
        screens.event.service_minutes.get() as i32,
    ) {
        e.start.set(e.chosen.get());
        screens
            .state
            .service_ui
            .box_shown
            .set(screens.state.service_ui.box_shown.get() ^ 1);
        screens.state.service_ui.box_page.set(e.page);
    }

    Ok(())
}

pub const GEAR_RATIO: Edit = Edit {
    page: 1,
    time: &crate::menu::owner::MENUS
        .screens
        .state
        .driver_stats
        .repair_times[12],
    chosen: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[0].chosen,
    start: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[0].start,
};
pub const TYRE: Edit = Edit {
    page: 2,
    time: &crate::menu::owner::MENUS
        .screens
        .state
        .driver_stats
        .repair_times[13],
    chosen: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[1].chosen,
    start: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[1].start,
};
pub const RIDE_HEIGHT: Edit = Edit {
    page: 5,
    time: &crate::menu::owner::MENUS
        .screens
        .state
        .driver_stats
        .repair_times[14],
    chosen: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[2].chosen,
    start: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[2].start,
};
pub const SUSPENSION_STIFFNESS: Edit = Edit {
    page: 6,
    time: &crate::menu::owner::MENUS
        .screens
        .state
        .driver_stats
        .repair_times[15],
    chosen: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[3].chosen,
    start: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[3].start,
};
