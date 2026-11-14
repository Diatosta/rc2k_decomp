//! The six edit pages of the service area: what the keys do while one of the setup values or the
//! tyre or gear ratio choice is being edited. Key 0x68 steps the value down, 0x69 up, 0x1C
//! confirms; every key also copies the clock into `last_key_time`.
//!
//! Steering sensitivity and brake balance change the car directly: confirming just leaves edit
//! mode. The other four cost service time; confirming either clears the flag (ride height and
//! stiffness whose chosen value is what the car has), flips the item's selection
//! (`toggle_change`, when the value was not changed in this edit), or clears the flag and asks
//! `try_toggle_repair` whether selecting it fits: status 2 puts the chosen value back to the
//! installed one, 1 selects and marks it (`choose_repair_marked`), 3 clears the marks and
//! selects it (`choose_repair`). All four end with `update_repair_total`.

use crate::errors::{BadInput, Refused};
use crate::input::take_key::take_key;
use crate::menu::owner::Menus;
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
use crate::system::owner::System;
use rc2k_core::menu::player::SetupField;

pub const KEY_CONFIRM: u8 = 0x1C;

/// A page whose value is changed in the car itself.
#[derive(Clone, Copy)]
pub struct SimplePage {
    pub down: Stepper,
    pub up: Stepper,
    /// The value in the player's car.
    pub in_car: SetupField,
}

/// A page whose value costs service time to change.
#[derive(Clone, Copy)]
pub struct ChangePage {
    pub down: Stepper,
    pub up: Stepper,
    pub change: Change,
    /// The car's value, for the pages that compare with it.
    pub in_car: Option<SetupField>,
    /// The page's menu item (the one to highlight).
    pub item: u32,
}

pub const STEERING_SENSITIVITY: SimplePage = SimplePage {
    down: step_steering_sensitivity_down::STEPPER,
    up: step_steering_sensitivity_up::STEPPER,
    in_car: SetupField::SteeringSensitivity,
};
pub const BRAKE_BALANCE: SimplePage = SimplePage {
    down: step_brake_balance_down::STEPPER,
    up: step_brake_balance_up::STEPPER,
    in_car: SetupField::BrakeBalance,
};
pub const GEAR_RATIO: ChangePage = ChangePage {
    down: step_gear_ratio_down::STEPPER,
    up: step_gear_ratio_up::STEPPER,
    change: toggle_gear_ratio_change::CHANGE,
    in_car: None,
    item: 0xE,
};
pub const TYRE: ChangePage = ChangePage {
    down: step_tyre_down::STEPPER,
    up: step_tyre_up::STEPPER,
    change: toggle_tyre_change::CHANGE,
    in_car: None,
    item: 0xF,
};
pub const RIDE_HEIGHT: ChangePage = ChangePage {
    down: step_ride_height_down::STEPPER,
    up: step_ride_height_up::STEPPER,
    change: toggle_ride_height_change::CHANGE,
    in_car: Some(SetupField::RideHeight),
    item: 0x12,
};
pub const SUSPENSION_STIFFNESS: ChangePage = ChangePage {
    down: step_suspension_stiffness_down::STEPPER,
    up: step_suspension_stiffness_up::STEPPER,
    change: toggle_suspension_stiffness_change::CHANGE,
    in_car: Some(SetupField::Stiffness),
    item: 0x13,
};

fn player(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, BadInput> {
    current_player(menus).ok_or(BadInput(
        "no current player object: the original reads through null",
    ))
}

/// The key, with the clock noted and the value stepped if it was an arrow. 0 = no key.
fn key_and_step(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    down: Stepper,
    up: Stepper,
) -> Result<u8, Refused> {
    use rc2k_core::service::setups::{PageKey, page_key};
    let key = take_key(input);
    let action = page_key(key);
    if action == PageKey::None {
        return Ok(0);
    }
    menus
        .play
        .champ_view
        .last_key_time
        .set(system.clocks().frame.get());
    match action {
        PageKey::Down => {
            step(menus, down)?;
        }
        PageKey::Up => {
            step(menus, up)?;
        }
        _ => {}
    }
    Ok(key)
}

pub fn simple_page(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    page: SimplePage,
) -> Result<(), Refused> {
    // Nothing may be written before it is known that nothing will be refused.
    player(menus)?;
    if { key_and_step(menus, system, input, page.down, page.up) }? == KEY_CONFIRM {
        leave_edit_mode(&menus.screens);
    }
    Ok(())
}

/// `confirm`: act as if the confirm key had been pressed, without taking a key (callers
/// outside the key handler pass true).
pub fn change_page(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    page: ChangePage,
    confirm: bool,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Everything the steps below could refuse is checked first, so that an error never
    // leaves half of the work done.
    let item = flagged_item(screens)?;
    if !has_repair_time(item) {
        return Err(BadInput("the highlighted item has no repair time").into());
    }
    chosen_repairs_time(menus)?;
    if !marks_ended() {
        return Err(BadInput("the second flag table has lost its end mark").into());
    }
    let installed_in_car = match page.in_car {
        Some(field) => Some((player(menus)?, field)),
        None => None,
    };

    if !confirm && key_and_step(menus, system, input, page.down, page.up)? != KEY_CONFIRM {
        return update_repair_total(menus);
    }

    use crate::service::state::{flags, marks, store_flags, store_marks, time_of};
    let (mut f, mut m) = (flags(), marks());
    let v = rc2k_core::service::repairs::Value {
        chosen: page.change.chosen(menus).get(),
        installed: page.change.installed(menus).get(),
        entry: page.change.start(menus).get(),
        in_car: installed_in_car.map(|(p, field)| field.get(p.get())),
    };
    let c = rc2k_core::service::repairs::confirm_change(
        &mut f,
        &mut m,
        item as usize,
        screens.event.service_minutes.get() as i32,
        v,
        |item| time_of(menus, item),
    )
    .map_err(|e| BadInput(e.0))?;
    store_flags(&f);
    store_marks(&m);
    page.change.chosen(menus).set(c.chosen);
    if let Some(status) = c.status {
        screens.state.service_ui.status.set(status);
    }
    screens.state.driver_stats.repair_time.set(c.total as u32);
    leave_edit_mode(screens);
    Ok(())
}
