//! The four item actions that open an edit page of the service area that costs service time
//! (gear ratio, tyre, ride height, suspension stiffness): if the highlighted item is chosen
//! already (its flag is 1), or the service time still covers the chosen repairs plus this
//! item's time (`budget - (total + time)`, signed, at least 0), the value at the edit's start is
//! the chosen one, edit mode flips and the edit page is the item's. Else nothing.
//!
//! Refuses (before anything) a highlighted item past the flag table.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::setup_values::setup_item;
use crate::service::state::{flagged_item, repair_flag_in};

/// One page's own variables.
#[derive(Clone, Copy)]
pub struct Edit {
    /// The page number written to `MenuState.service_ui.box_page`.
    pub page: u32,
    /// The setup item (0 .. 3): its service time is
    /// `MenuState.driver_stats.repair_times[12 + setup]`, its chosen value is copied to the
    /// value at the edit's start.
    pub setup: usize,
}

pub fn begin_edit(menus: &Menus, e: Edit) -> Result<(), Refused> {
    let screens = &menus.screens;
    let item = flagged_item(screens)?;

    if rc2k_core::service::repairs::may_edit(
        repair_flag_in(&menus.tables, item),
        screens.state.driver_stats.repair_time.get() as i32,
        screens.state.driver_stats.repair_times[12 + e.setup].get() as i32,
        screens.event.service_minutes.get() as i32,
    ) {
        let item = &setup_item(menus, e.setup);
        item.start.set(item.chosen.get());
        screens
            .state
            .service_ui
            .box_shown
            .set(!screens.state.service_ui.box_shown.is_set());
        screens.state.service_ui.box_page.set(e.page);
    }

    Ok(())
}

pub const GEAR_RATIO: Edit = Edit { page: 1, setup: 0 };
pub const TYRE: Edit = Edit { page: 2, setup: 1 };
pub const RIDE_HEIGHT: Edit = Edit { page: 5, setup: 2 };
pub const SUSPENSION_STIFFNESS: Edit = Edit { page: 6, setup: 3 };
