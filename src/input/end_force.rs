//! `input::end_force` (0x4095AE): a device's force feedback ended, when its controller has
//! it (the controller's entry, `g_controllers` + 0x50 its number, +0x1C 1): the
//! self-centring turned on (`DIPROP_AUTOCENTER` 1 in `dword_4F6AA4`; its result not looked
//! at) and the six effects' objects given back (`input::release_effect`), each effect's
//! +0xC (found) 0. (+0x28, effects on, is left: `input::release_device` clears it.)
//!
//! In:  esi = the device's record (+4 its DirectInput device)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::release_device`. The controller's number is not checked (as the
//! original's). Shadow mode runs the original alone (it calls DirectInput). Self-test: the
//! original and the Rust version on DirectInput objects made in Rust (`input::di_harness`)
//! and a copy of the first device's record whose effects were made (`input::begin_force`):
//! a controller with no force feedback, the wheel.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::field;
use crate::input::devices::{at, put};
use crate::input::di_harness::{EFFECT_LEN, EFFECTS_AT, entry_of, method, slot};
use crate::input::types::{Controller, ForceEffect, InputDevice};

/// The Rust version (see the header).
pub fn end_force(record: u32) {
    if at(entry_of(record), field!(Controller, force_feedback)) != 1 {
        return;
    }
    let prop = crate::input::begin_force::dword_property(1);
    method(
        at(record, field!(InputDevice, device)),
        slot::SET_PROPERTY,
        &[9, prop],
    );
    for n in 0..6 {
        let e = record.wrapping_add(EFFECTS_AT + n * EFFECT_LEN);
        crate::input::effect_objects::release_effect(e);
        put(e, field!(ForceEffect, found), 0);
    }
}
