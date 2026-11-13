//! The controller set-up pages' item actions and set-ups (`menu::player1_controller_loop` and
//! the seven after it): what they change of the current player's record: +0x2C the controller
//! (0 none, 1 .. the controller count the listed ones), +0x34 its sensitivity (0 .. 3), +0x38
//! its mode, +0x40 and +0x44 the two dead zones (their sum held at 90).
//! - apply / release the controller (`input::request_device_setup` / `_release`).
//! - next / previous controller: up (after the count, 0) or down (0 or less: the count); then
//!   the release with 0, else the set-up.
//! - lower / raise the sensitivity (0 .. 3, signed) and the set-up. Raising reads the dword at
//!   +0x2C of the item or menu the caller hands on before the player's record is taken (kept).
//! - raise the dead zones: +0x40 (or +0x44) one more if `100 - (+0x40) - (+0x44)` is above 10.
//!
//! Refusals (before anything): no current player record; a set-up to make without a first
//! input device record; for `raise_controller_sensitivity` an item or menu address below
//! 0x10000.

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
pub fn apply_controller(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let p = player(menus)?;
    let c = p.get().controller;
    if would_apply(input, c)? {
        set_up_device(input, p, c)?;
    }
    Ok(())
}

/// `menu::release_controller`.
pub fn release_controller(input: &Input) {
    request_device_release(input);
}

/// The device's set-up with the player's sensitivity and pedal axes.
fn set_up_device(input: &Input, p: PlayerCell, c: u32) -> Result<(), Refused> {
    let q = p.get();
    let axes = u32::from_le_bytes(q.axis_codes);
    { request_device_setup(c - 1, q.sensitivity_level, axes, 0, 0, input) }?;
    Ok(())
}

/// The release for 0, else the set-up; refused (before anything) as the set-up is.
fn choose(input: &Input, p: PlayerCell, c: u32) -> Result<(), Refused> {
    let applies = if c == 0 {
        false
    } else {
        would_apply(input, c)?
    };
    p.update(|q| q.controller = c);
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
    choose(input, p, c)
}

/// `menu::previous_controller`.
pub fn previous_controller(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let p = player(menus)?;
    let c = rc2k_core::menu::steps::previous_controller(p.get().controller, count(input));
    choose(input, p, c)
}

/// The sensitivity to `value` and the set-up, refused (before anything) as the set-up is.
fn set_sensitivity(input: &Input, p: PlayerCell, value: u32) -> Result<(), Refused> {
    let c = p.get().controller;
    let applies = would_apply(input, c)?;
    p.update(|q| q.sensitivity_level = value);
    if applies {
        set_up_device(input, p, c)?;
    }
    Ok(())
}

/// `menu::lower_controller_sensitivity`.
pub fn lower_controller_sensitivity(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let p = player(menus)?;
    if let Some(s) =
        rc2k_core::menu::steps::lower_sensitivity(p.get().controller, p.get().sensitivity_level)
    {
        { set_sensitivity(input, p, s) }?;
    }
    Ok(())
}

/// `menu::raise_controller_sensitivity`. `gate` is the dword at +0x2C of what the caller hands
/// on: a key's item (its left link) or an action's menu (its first item's down link).
pub fn raise_controller_sensitivity(
    menus: &Menus,
    input: &Input,
    gate: u32,
) -> Result<(), Refused> {
    if gate == 0 {
        return Ok(());
    }
    let p = player(menus)?;
    if let Some(s) = rc2k_core::menu::steps::raise_sensitivity(p.get().sensitivity_level) {
        { set_sensitivity(input, p, s) }?;
    }
    Ok(())
}

/// `menu::raise_dead_zone_a` (the first) / `raise_dead_zone_b`: the dead zone one more if
/// the two leave more than 10.
pub fn raise_dead_zone(menus: &Menus, second: bool) -> Result<(), Refused> {
    let p = player(menus)?;
    let q = p.get();
    let (_, grows) = rc2k_core::menu::steps::dead_zone_room(q.end_zone_pct, q.centre_dead_zone_pct);
    if grows {
        p.update(|q| {
            let zone = if second {
                &mut q.centre_dead_zone_pct
            } else {
                &mut q.end_zone_pct
            };
            *zone = zone.wrapping_add(1);
        });
    }
    Ok(())
}
