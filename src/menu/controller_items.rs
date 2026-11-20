//! The controller page's item actions and set-ups (the controller pages of `menu::player_pages`):
//! what they change of the current player's record: the controller (0 none, 1 .. the controller
//! count the listed ones).
//! - apply / release the controller (`input::request_device_setup` / `_release`).
//! - next / previous controller: up (after the count, 0) or down (0 or less: the count); then
//!   the release with 0, else the set-up.
//!
//! Refusals (before anything): no current player record; a set-up to make without a first
//! input device record.

use crate::errors::{BadInput, Refused};
use crate::input::owner::Input;
use crate::input::request_device_release::request_device_release;
use crate::input::request_device_setup::request_device_setup;
use crate::menu::owner::Menus;
use crate::menu::player_settings::current;
use crate::menu::standings::PlayerCell;

fn player(menus: &Menus) -> Result<PlayerCell<'_>, Refused> {
    current(menus)
}

fn count(input: &Input) -> u32 {
    input.devices.controller_count.get()
}

/// Whether the set-up would ask for the device; refused without a record to ask with.
fn would_apply(input: &Input, controller: u32) -> Result<bool, Refused> {
    let applies = rc2k_core::menu::steps::listed(controller, count(input));
    if applies && input.devices.device(0).is_none() {
        return Err(BadInput("no first input device record").into());
    }
    Ok(applies)
}

/// `menu::apply_controller`.
pub(super) fn apply_controller(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let p = player(menus)?;
    let c = p.get().controller;
    if would_apply(input, c)? {
        set_up_device(input, p, c)?;
    }
    Ok(())
}

/// `menu::release_controller`.
pub(super) fn release_controller(input: &Input) {
    request_device_release(input);
}

/// The device's set-up with the player's sensitivity and pedal axes.
fn set_up_device(input: &Input, p: PlayerCell, c: u32) -> Result<(), Refused> {
    let q = p.get();
    let axes = u32::from_le_bytes(q.axis_codes);
    { request_device_setup(c - 1, q.sensitivity_level, axes, 0, 0, input) }?;
    Ok(())
}

/// The release for 0, else the set-up; refused (before anything) as the set-up is. The choice
/// is kept by what the controller is (`input::device_ids`).
fn choose(menus: &Menus, input: &Input, p: PlayerCell, c: u32) -> Result<(), Refused> {
    let applies = if c == 0 {
        false
    } else {
        would_apply(input, c)?
    };
    p.update(|q| q.controller = c);
    crate::input::device_ids::note_choice(input, menus.screens.event.current_player.get(), c);
    if c == 0 {
        release_controller(input);
    } else if applies {
        set_up_device(input, p, c)?;
    }
    Ok(())
}

/// `menu::next_controller`.
pub fn next_controller(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let p = player(menus)?;
    let c = rc2k_core::menu::steps::next_controller(p.get().controller, count(input));
    choose(menus, input, p, c)
}

/// `menu::previous_controller`.
pub fn previous_controller(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let p = player(menus)?;
    let c = rc2k_core::menu::steps::previous_controller(p.get().controller, count(input));
    choose(menus, input, p, c)
}
