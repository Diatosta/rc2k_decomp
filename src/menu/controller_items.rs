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
use crate::framework::{EAX, EBX, EDI, RawHook, Regs, SelfTest, game_read, game_write, rva};
use crate::input::input_tick::COMMAND_RVA;
use crate::input::request_device_release::request_device_release;
use crate::input::request_device_setup::{DEVICES_RVA, request_device_setup};
use crate::menu::player::{PLAYERS_RVA, Player};
use crate::menu::player_settings::current;
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::state::PLAYER_INDEX_RVA;

pub const COUNT_RVA: usize = rva(0x4F78E0); // dword_4F78E0: the listed controllers

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

/// What they may write: the current player's record, the device record's first bytes and
/// the command.
pub fn regions() -> Vec<Region> {
    let mut r = crate::input::request_device_setup::regions();
    if let Ok(p) = player() {
        r.push((p, std::mem::size_of::<Player>()));
    }
    r
}

/// The dispatch of each: `rust` the Rust version as the harness runs it.
pub fn dispatch_item(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
) {
    dispatch_state(hook, r, call_original, &regions(), rust);
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

/// The self-test of each: a player record of the test's in player 0's place (the current
/// player 0), a device record of the test's, controllers 0 .. 4 with counts 0, 2 and 3,
/// sensitivities -1 .. 4, dead zones around their limit, the caller's edi on a dword 0 and 1.
pub fn test_item(
    hook: &RawHook,
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let inputs = [
        at(PLAYER_INDEX_RVA),
        at(PLAYERS_RVA),
        at(DEVICES_RVA),
        at(COMMAND_RVA),
        at(COUNT_RVA),
    ];
    let saved = capture(&inputs);
    let mut record = vec![0u8; std::mem::size_of::<Player>()];
    let mut device = vec![0u8; 0x40];
    let mut caller = [0u32; 0x10];
    let (p, d, e) = (
        record.as_mut_ptr() as u32,
        device.as_mut_ptr() as u32,
        caller.as_mut_ptr() as u32,
    );
    let (record_len, device_len) = (record.len(), device.len());
    let regions: [Region; 3] = [at(COMMAND_RVA), (p, record_len), (d, device_len)];
    // controller, count, sensitivity, dead zones +0x40 and +0x44
    let cases: [(u32, u32, u32, u32, u32); 12] = [
        (0, 2, 0, 10, 20),
        (1, 2, 1, 40, 39),
        (2, 2, 3, 40, 40),
        (3, 2, 2, 45, 45),
        (2, 3, 0xFFFF_FFFF, 0, 89),
        (3, 3, 4, 0, 90),
        (0, 0, 2, 50, 41),
        (1, 0, 2, 0, 0),
        (0xFFFF_FFFF, 2, 1, 5, 5),
        (4, 3, 1, 100, 0),
        (2, 2, 2, 0xFFFF_FFF0, 0),
        (1, 2, 0, 0x7FFF_FFFF, 0x7FFF_FFFF),
    ];
    for (case, &(controller, n, sensitivity, zone_a, zone_b)) in cases.iter().enumerate() {
        for edi_word in [0u32, 1] {
            let before = Regs {
                edi: e,
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!(
                    "case {case}: controller {controller:#x} of {n}, sensitivity {sensitivity:#x}, dead zones {zone_a:#x} {zone_b:#x}, caller's +0x2C {edi_word}"
                ),
                call_original,
                before,
                &|| unsafe {
                    for (k, b) in std::slice::from_raw_parts_mut(p as *mut u8, record_len)
                        .iter_mut()
                        .enumerate()
                    {
                        *b = (k as u8).wrapping_mul(7);
                    }
                    std::ptr::write_bytes(d as *mut u8, 0x5A, device_len);
                    // No DirectInput device and not acquired, should the input thread act
                    // on a request before it is put back.
                    w(d, 4, 0);
                    w(d, 0x24, 0);
                    w(p, 0x2C, controller);
                    w(p, 0x34, sensitivity);
                    w(p, 0x38, 0x77 + case as u32);
                    w(p, 0x40, zone_a);
                    w(p, 0x44, zone_b);
                    w(e, 0x2C, edi_word);
                    game_write(PLAYER_INDEX_RVA, 0u32);
                    game_write(PLAYERS_RVA, p);
                    game_write(DEVICES_RVA, d);
                    game_write(COMMAND_RVA, 0u32);
                    game_write(COUNT_RVA, n);
                },
                &regions,
                rust,
            );
        }
    }
    restore(&inputs, &saved);
    drop((record, device, caller));
    t.finish()
}
