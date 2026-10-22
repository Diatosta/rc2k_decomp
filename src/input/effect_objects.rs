//! A force-feedback effect's DirectInput object (`IDirectInputEffect`, an effect's +0x30;
//! the effects are a device record's six from +0x3C, 0xA4 bytes each), asked by the game's
//! four small functions, each doing nothing when the effect has no object:
//! - `input::release_effect` (0x409ACC): `Stop`, `Release`, +0x30 0;
//! - `input::start_effect` (0x409AEE): `Download`, `Start` (one iteration, no flags);
//! - `input::set_effect_parameters` (0x409B1F): `SetParameters` with the effect's
//!   `DIEFFECT` (+0x34) and eax the flags;
//! - `input::stop_effect` (0x409B0D): `Stop`.
//!
//! In:  edi = the effect, eax = the flags (`set_effect_parameters`)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the effects' functions (`input::effect_N_change`, through
//! `effect_changes::game_part`), `input::make_effect` and `input::end_force`
//! (`release_effect`). The methods are the object's own (DirectInput's in the game, Rust's
//! in the standalone program, `input::di_objects`). Shadow mode runs the original alone (it
//! calls DirectInput). Self-test: the original and the Rust version on DirectInput objects
//! made in Rust (`input::di_harness`): an effect with no object and one with a constant
//! force's, `set_effect_parameters` with several flags.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::field;
use crate::input::devices::{at, put};
use crate::input::di_harness::{EFFECT_LEN, method, slot};
use crate::input::types::ForceEffect;

/// `release_effect`'s Rust version.
pub fn release_effect(effect: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(object, slot::STOP, &[]);
        method(object, slot::RELEASE, &[]);
        put(effect, field!(ForceEffect, object), 0);
    }
}

/// `start_effect`'s Rust version.
pub fn start_effect(effect: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(object, slot::DOWNLOAD, &[]);
        method(object, slot::START, &[1, 0]);
    }
}

/// `set_effect_parameters`'s Rust version.
pub fn set_effect_parameters(effect: u32, flags: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(
            object,
            slot::SET_PARAMETERS,
            &[effect.wrapping_add(field!(ForceEffect, effect)), flags],
        );
    }
}

/// `stop_effect`'s Rust version.
pub fn stop_effect(effect: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(object, slot::STOP, &[]);
    }
}

/// An effect of the test's at `entry` (0xA4 bytes): a constant force's parameters
/// (`input::make_effect`'s), and an object made for it on the wheel (`object`).
pub fn test_effect(entry: u32, object: bool) {
    unsafe { std::ptr::write_bytes(entry as *mut u8, 0x5A, EFFECT_LEN as usize) };
    put(entry, field!(ForceEffect, kind), 1);
    put(entry, field!(ForceEffect, found), 1);
    put(entry, field!(ForceEffect, object), 0);
    let constant = crate::input::di_objects::guid(
        0x1354_1C20,
        0x8E33,
        0x11D0,
        [0x9A, 0xD0, 0x00, 0xA0, 0xC9, 0xA0, 0x6E, 0x35],
    );
    unsafe {
        std::ptr::copy_nonoverlapping(
            constant.as_ptr(),
            (entry + field!(ForceEffect, guid)) as *mut u8,
            16,
        )
    };
    crate::input::make_effect::fill_parameters(entry);
    if object {
        let device = crate::input::di_objects::test_device(1);
        method(
            device,
            slot::CREATE_EFFECT,
            &[
                entry + field!(ForceEffect, guid),
                entry + field!(ForceEffect, effect),
                entry + field!(ForceEffect, object),
                0,
            ],
        );
    }
}
