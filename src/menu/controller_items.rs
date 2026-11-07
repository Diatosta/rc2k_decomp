//! The controller set-up pages' item actions and set-ups (`menu::player1_controller_loop` and
//! the seven after it): what they change of the current player's record (`dword_75598C`,
//! `g_players`): +0x2C the controller (0 none, 1 .. `dword_4F78E0` the listed ones), +0x34
//! its sensitivity (0 .. 3), +0x38 its mode, +0x40 and +0x44 the two dead zones (their sum
//! held at 90).
//! - `menu::apply_controller` (0x46B47C): with a listed controller (`controller - 1` below
//!   the count, unsigned), `input::request_device_setup` with it less 1, +0x34, +0x38, edx and
//!   ebp 0. Every register kept.
//! - `menu::release_controller` (0x46B4B4): `input::request_device_release`.
//! - `menu::next_controller` (0x46A3BF), `previous_controller` (0x46A40B): the controller
//!   up (after the count, 0) or down (0 or less: the count); then with 0 the release, else
//!   the set-up.
//! - `menu::lower_controller_sensitivity` (0x46B414): with a controller and +0x34 above 0
//!   (signed), one less and the set-up.
//! - `menu::raise_controller_sensitivity` (0x46B44F): if the dword at +0x2C of the caller's
//!   edi (the original's slip: it reads it before taking the player's record; edi is what
//!   `menu::run_item_action` hands on) is not 0 and +0x34 is below 3 (signed), one more
//!   and the set-up.
//! - `menu::raise_dead_zone_a` (0x46B3A3), `raise_dead_zone_b` (0x46B3EA): +0x40 (or +0x44)
//!   one more if `100 - (+0x40) - (+0x44)` is above 10 (signed). The first is not
//!   `pusha`'d, but no caller reads the eax, ebx and edi it leaves.
//!
//! Refusals (before anything): no current player record; a set-up to make without a
//! first input device record; for `raise_controller_sensitivity` an edi below 0x10000.

use crate::errors::{BadInput, Refused};
use crate::input::owner::input;
use crate::input::request_device_release::request_device_release;
use crate::input::request_device_setup::request_device_setup;
use crate::menu::player_settings::current;
use crate::menu::standings::PlayerCell;

fn player() -> Result<PlayerCell, Refused> {
    current()
}

fn count() -> u32 {
    input().devices.controller_count.get()
}

/// Whether the set-up would ask for the device; refused without a record to ask with.
fn would_apply(controller: u32) -> Result<bool, Refused> {
    let applies = rc2k_core::menu::steps::listed(controller, count());
    if applies && input().devices.device(0).is_none() {
        return Err(BadInput("no first input device record").into());
    }
    Ok(applies)
}

/// `menu::apply_controller`.
pub fn apply_controller() -> Result<(), Refused> {
    let p = player()?;
    let c = p.get().controller;
    if would_apply(c)? {
        set_up_device(p, c)?;
    }
    Ok(())
}

/// `menu::release_controller`.
pub fn release_controller(input: &crate::input::owner::Input) {
    request_device_release(input);
}

/// The device's set-up with the player's sensitivity and pedal axes.
fn set_up_device(p: PlayerCell, c: u32) -> Result<(), Refused> {
    let input = crate::input::owner::input();
    let q = p.get();
    let axes = u32::from_le_bytes(q.axis_codes);
    { request_device_setup(c - 1, q.sensitivity_level, axes, 0, 0, input) }?;
    Ok(())
}

/// The release for 0, else the set-up; refused (before anything) as the set-up is.
fn choose(p: PlayerCell, c: u32) -> Result<(), Refused> {
    let input = crate::input::owner::input();
    let applies = if c == 0 { false } else { would_apply(c)? };
    p.update(|q| q.controller = c);
    if c == 0 {
        release_controller(input);
    } else if applies {
        set_up_device(p, c)?;
    }
    Ok(())
}

/// `menu::next_controller`.
pub fn next_controller() -> Result<(), Refused> {
    let p = player()?;
    let c = rc2k_core::menu::steps::next_controller(p.get().controller, count());
    choose(p, c)
}

/// `menu::previous_controller`.
pub fn previous_controller() -> Result<(), Refused> {
    let p = player()?;
    let c = rc2k_core::menu::steps::previous_controller(p.get().controller, count());
    choose(p, c)
}

/// The sensitivity to `value` and the set-up, refused (before anything) as the set-up is.
fn set_sensitivity(p: PlayerCell, value: u32) -> Result<(), Refused> {
    let c = p.get().controller;
    let applies = would_apply(c)?;
    p.update(|q| q.sensitivity_level = value);
    if applies {
        set_up_device(p, c)?;
    }
    Ok(())
}

/// `menu::lower_controller_sensitivity`.
pub fn lower_controller_sensitivity() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::steps::lower_sensitivity`.
    let p = player()?;
    if let Some(s) =
        rc2k_core::menu::steps::lower_sensitivity(p.get().controller, p.get().sensitivity_level)
    {
        { set_sensitivity(p, s) }?;
    }
    Ok(())
}

/// `menu::raise_controller_sensitivity`; `edi` is the caller's.
/// `gate` is the dword at +0x2C of what edi points at: a key's item (its left link) or an
/// action's menu (its first item's down link).
pub fn raise_controller_sensitivity(gate: u32) -> Result<(), Refused> {
    if gate == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::menu::steps::raise_sensitivity`.
    let p = player()?;
    if let Some(s) = rc2k_core::menu::steps::raise_sensitivity(p.get().sensitivity_level) {
        { set_sensitivity(p, s) }?;
    }
    Ok(())
}

/// `menu::raise_dead_zone_a` (the first) / `raise_dead_zone_b`: the dead zone one more if
/// the two leave more than 10.
pub fn raise_dead_zone(second: bool) -> Result<(), Refused> {
    let p = player()?;
    let q = p.get();
    let (_, grows) = rc2k_core::menu::steps::dead_zone_room(q.dead_zone_1, q.dead_zone_2);
    if grows {
        p.update(|q| {
            let zone = if second {
                &mut q.dead_zone_2
            } else {
                &mut q.dead_zone_1
            };
            *zone = zone.wrapping_add(1);
        });
    }
    Ok(())
}
