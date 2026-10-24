//! The controller set-up pages' item actions and set-ups (`menu::screen_47814c_loop` and
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
//!   `pusha`'d: it leaves eax the value (one more or not), ebx `100 - ...`, edi the record.
//!
//! Refusals (before anything): no current player record; a set-up to make without a
//! first input device record; for `raise_controller_sensitivity` an edi below 0x10000.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, EDI};
use crate::input::request_device_release::request_device_release;
use crate::input::request_device_setup::{DEVICES_RVA, request_device_setup};
use crate::input::store::DeviceSlot;
use crate::menu::player_settings::current;
use crate::screens::harness::Left;
use crate::store::game_read;

pub const COUNT_RVA: DeviceSlot = DeviceSlot::at(0x4F78E0); // dword_4F78E0: the listed controllers

fn f(p: u32, o: u32) -> u32 {
    unsafe { (p.wrapping_add(o) as *const u32).read_unaligned() }
}

fn w(p: u32, o: u32, v: u32) {
    unsafe { (p.wrapping_add(o) as *mut u32).write_unaligned(v) }
}

fn player() -> Result<u32, Refused> {
    Ok(current()? as u32)
}

fn count() -> u32 {
    unsafe { game_read(COUNT_RVA) }
}

/// Whether the set-up would ask for the device; refused without a record to ask with.
fn would_apply(controller: u32) -> Result<bool, Refused> {
    let applies = rc2k_core::menu::steps::listed(controller, count());
    if applies && unsafe { game_read::<u32>(DEVICES_RVA) } == 0 {
        return Err(BadInput("no first input device record").into());
    }
    Ok(applies)
}

/// `menu::apply_controller`.
pub unsafe fn apply_controller() -> Result<(), Refused> {
    let p = player()?;
    let c = f(p, 0x2C);
    if would_apply(c)? {
        unsafe { request_device_setup(c - 1, f(p, 0x34), f(p, 0x38), 0, 0) }?;
    }
    Ok(())
}

/// `menu::release_controller`.
pub fn release_controller() {
    request_device_release();
}

/// The release for 0, else the set-up; refused (before anything) as the set-up is.
unsafe fn choose(p: u32, c: u32) -> Result<(), Refused> {
    let applies = if c == 0 { false } else { would_apply(c)? };
    w(p, 0x2C, c);
    if c == 0 {
        release_controller();
    } else if applies {
        unsafe { request_device_setup(c - 1, f(p, 0x34), f(p, 0x38), 0, 0) }?;
    }
    Ok(())
}

/// `menu::next_controller`.
pub unsafe fn next_controller() -> Result<(), Refused> {
    let p = player()?;
    let c = f(p, 0x2C);
    let c = rc2k_core::menu::steps::next_controller(c, count());
    unsafe { choose(p, c) }
}

/// `menu::previous_controller`.
pub unsafe fn previous_controller() -> Result<(), Refused> {
    let p = player()?;
    let c = f(p, 0x2C);
    let c = rc2k_core::menu::steps::previous_controller(c, count());
    unsafe { choose(p, c) }
}

/// +0x34 to `value` and the set-up, refused (before anything) as the set-up is.
unsafe fn set_sensitivity(p: u32, value: u32) -> Result<(), Refused> {
    let c = f(p, 0x2C);
    let applies = would_apply(c)?;
    w(p, 0x34, value);
    if applies {
        unsafe { request_device_setup(c - 1, value, f(p, 0x38), 0, 0) }?;
    }
    Ok(())
}

/// `menu::lower_controller_sensitivity`.
pub unsafe fn lower_controller_sensitivity() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::steps::lower_sensitivity`.
    let p = player()?;
    if let Some(s) = rc2k_core::menu::steps::lower_sensitivity(f(p, 0x2C), f(p, 0x34)) {
        unsafe { set_sensitivity(p, s) }?;
    }
    Ok(())
}

/// `menu::raise_controller_sensitivity`; `edi` is the caller's.
pub unsafe fn raise_controller_sensitivity(edi: u32) -> Result<(), Refused> {
    if edi < 0x10000 {
        return Err(BadInput("an edi that points nowhere (the original reads +0x2C of it)").into());
    }
    if f(edi, 0x2C) == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::menu::steps::raise_sensitivity`.
    let p = player()?;
    if let Some(s) = rc2k_core::menu::steps::raise_sensitivity(f(p, 0x34)) {
        unsafe { set_sensitivity(p, s) }?;
    }
    Ok(())
}

/// A dead zone (`offset` 0x40 or 0x44) one more if the two leave more than 10; returns
/// (the value, `100 - both`, the record) as the registers have them.
pub fn raise_dead_zone(offset: u32) -> Result<(u32, u32, u32), Refused> {
    let p = player()?;
    let mut value = f(p, offset);
    let (room, grows) = rc2k_core::menu::steps::dead_zone_room(f(p, 0x40), f(p, 0x44));
    if grows {
        value = value.wrapping_add(1);
        w(p, offset, value);
    }
    Ok((value, room, p))
}

/// What `raise_dead_zone_a` leaves.
pub fn dead_zone_left(offset: u32) -> Result<Left, Refused> {
    let (value, room, p) = raise_dead_zone(offset)?;
    Ok(if offset == 0x40 {
        Left::nothing()
            .with(EAX, value)
            .with(EBX, room)
            .with(EDI, p)
    } else {
        Left::nothing()
    })
}
