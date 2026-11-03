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
//! in the standalone program, `input::di_objects`).

use crate::input::types::ForceEffect;
use crate::pointers::GamePtr;

/// `release_effect`'s Rust version.
pub fn release_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::stop(object);
        crate::input::di_objects::release(object);
        effect.object.set(0);
    }
}

/// `start_effect`'s Rust version.
pub fn start_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::download(object);
        crate::input::di_objects::start(object, 1, 0);
    }
}

/// `set_effect_parameters`'s Rust version.
pub fn set_effect_parameters(effect: &ForceEffect, flags: u32) {
    let object = effect.object.get();
    if object != 0 {
        // The `DIEFFECT` is handed on by address (the DirectInput stand-in reads it).
        crate::input::di_objects::set_parameters(object, GamePtr::to(&effect.effect).addr(), flags);
    }
}

/// `stop_effect`'s Rust version.
pub fn stop_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::stop(object);
    }
}
