//! The six edit pages of the service area: what the keys do while one of the setup values or
//! the tyre or gear ratio choice is being edited ("locked", in the original's error text).
//! Key 0x68 steps the value down, 0x69 up, 0x1C confirms. Every key also copies
//! `dword_51899C` (a clock) into `dword_7556C4`.
//!
//! Two pages (steering sensitivity, brake balance) change the car directly: confirming just
//! leaves edit mode. The other four cost service time; confirming goes through this:
//!   - ride height and stiffness only: if the chosen value is what the car has, the item's
//!     flag is cleared and that is all;
//!   - if the value was not changed in this edit (it equals the value at entry), the item's
//!     selection is flipped (`toggle_change`), and unless that selected it, that is all;
//!   - the flag is cleared and `try_toggle_repair` says whether selecting it fits:
//!     - status 2 (no time left at all): the chosen value goes back to the installed one;
//!     - status 1 (fits only by overrunning): selected and marked (`choose_repair_marked`);
//!     - status 3 (fits): the marks are cleared and it is selected (`choose_repair`).
//!
//! Whatever happened, the four end with `update_repair_total`.

use crate::errors::{BadInput, Refused};
use crate::input::take_key::take_key;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::state_store::MENU_STATE;
use crate::service::setup_values::{Change, Stepper, step};
use crate::service::state::{
    chosen_repairs_time, current_player, flagged_item, has_repair_time, leave_edit_mode,
    marks_ended,
};
use crate::service::update_repair_total::update_repair_total;
use crate::service::{
    step_brake_balance_down, step_brake_balance_up, step_gear_ratio_down, step_gear_ratio_up,
    step_ride_height_down, step_ride_height_up, step_steering_sensitivity_down,
    step_steering_sensitivity_up, step_suspension_stiffness_down, step_suspension_stiffness_up,
    step_tyre_down, step_tyre_up, toggle_gear_ratio_change, toggle_ride_height_change,
    toggle_suspension_stiffness_change, toggle_tyre_change,
};
use crate::session::{CLOCKS, Dword};

pub const KEY_CONFIRM: u8 = 0x1C;

/// A page whose value is changed in the car itself.
#[derive(Clone, Copy)]
pub struct SimplePage {
    pub down: Stepper,
    pub up: Stepper,
    /// Offset of the value in the player.
    pub car_offset: u32,
}

/// A page whose value costs service time to change.
#[derive(Clone, Copy)]
pub struct ChangePage {
    pub down: Stepper,
    pub up: Stepper,
    pub change: Change,
    /// The chosen value as it was when the edit started.
    pub entry: &'static Dword,
    /// Where the car has this value, for the pages that compare with it.
    pub car_offset: Option<u32>,
    /// The page's menu item (the self-tests highlight it).
    pub item: u32,
}

pub const STEERING_SENSITIVITY: SimplePage = SimplePage {
    down: step_steering_sensitivity_down::STEPPER,
    up: step_steering_sensitivity_up::STEPPER,
    car_offset: 0x11C,
};
pub const BRAKE_BALANCE: SimplePage = SimplePage {
    down: step_brake_balance_down::STEPPER,
    up: step_brake_balance_up::STEPPER,
    car_offset: 0x110,
};
pub const GEAR_RATIO: ChangePage = ChangePage {
    down: step_gear_ratio_down::STEPPER,
    up: step_gear_ratio_up::STEPPER,
    change: toggle_gear_ratio_change::CHANGE,
    entry: &MENU_STATE.driver_stats.setups[0].start,
    car_offset: None,
    item: 0xE,
};
pub const TYRE: ChangePage = ChangePage {
    down: step_tyre_down::STEPPER,
    up: step_tyre_up::STEPPER,
    change: toggle_tyre_change::CHANGE,
    entry: &MENU_STATE.driver_stats.setups[1].start,
    car_offset: None,
    item: 0xF,
};
pub const RIDE_HEIGHT: ChangePage = ChangePage {
    down: step_ride_height_down::STEPPER,
    up: step_ride_height_up::STEPPER,
    change: toggle_ride_height_change::CHANGE,
    entry: &MENU_STATE.driver_stats.setups[2].start,
    car_offset: Some(0x114),
    item: 0x12,
};
pub const SUSPENSION_STIFFNESS: ChangePage = ChangePage {
    down: step_suspension_stiffness_down::STEPPER,
    up: step_suspension_stiffness_up::STEPPER,
    change: toggle_suspension_stiffness_change::CHANGE,
    entry: &MENU_STATE.driver_stats.setups[3].start,
    car_offset: Some(0x118),
    item: 0x13,
};

fn player() -> Result<crate::menu::standings::PlayerCell, BadInput> {
    current_player().ok_or(BadInput(
        "no current player object: the original reads through null",
    ))
}

/// The key, with the clock noted and the value stepped if it was an arrow. 0 = no key.
fn key_and_step(down: Stepper, up: Stepper) -> Result<u8, Refused> {
    // Converted: `rc2k_core::service::setups::page_key`.
    use rc2k_core::service::setups::{PageKey, page_key};
    let key = take_key();
    let action = page_key(key);
    if action == PageKey::None {
        return Ok(0);
    }
    CHAMP_VIEW.last_key_time.set(CLOCKS.frame.get());
    match action {
        PageKey::Down => {
            step(down)?;
        }
        PageKey::Up => {
            step(up)?;
        }
        _ => {}
    }
    Ok(key)
}

pub fn simple_page(page: SimplePage) -> Result<(), Refused> {
    // Nothing may be written before it is known that nothing will be refused.
    player()?;
    if { key_and_step(page.down, page.up) }? == KEY_CONFIRM {
        leave_edit_mode();
    }
    Ok(())
}

/// `confirm`: act as if the confirm key had been pressed, without taking a key (the callers
/// outside the key handler pass ebx = 1 for this).
pub fn change_page(page: ChangePage, confirm: bool) -> Result<(), Refused> {
    // Everything the steps below could refuse is checked first, so that an error never
    // leaves half of the work done.
    let item = flagged_item()?;
    if !has_repair_time(item) {
        return Err(BadInput("the highlighted item has no repair time").into());
    }
    chosen_repairs_time()?;
    if !marks_ended() {
        return Err(BadInput("the second flag table has lost its end mark").into());
    }
    let installed_in_car = match page.car_offset {
        Some(offset) => Some((player()?, offset as usize)),
        None => None,
    };

    if !confirm && key_and_step(page.down, page.up)? != KEY_CONFIRM {
        return update_repair_total();
    }

    // Converted: `rc2k_core::service::repairs::confirm_change`.
    use crate::service::state::{flags, marks, store_flags, store_marks, time_of};
    let (mut f, mut m) = (flags(), marks());
    let v = rc2k_core::service::repairs::Value {
        chosen: page.change.chosen.get(),
        installed: page.change.installed.get(),
        entry: page.entry.get(),
        in_car: installed_in_car.map(|(p, at)| p.get().word(at)),
    };
    let c = rc2k_core::service::repairs::confirm_change(
        &mut f,
        &mut m,
        item as usize,
        EVENT_STATE.service_minutes.get() as i32,
        v,
        time_of,
    )
    .map_err(|e| BadInput(e.0))?;
    store_flags(&f);
    store_marks(&m);
    page.change.chosen.set(c.chosen);
    if let Some(status) = c.status {
        MENU_STATE.service_ui.status.set(status);
    }
    MENU_STATE.driver_stats.repair_time.set(c.total as u32);
    leave_edit_mode();
    Ok(())
}

// ---- hook side ----

// ---- self-tests ----
