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

use crate::errors::BadInput;
use crate::field;
use crate::input::devices::{at, put};
use crate::input::types::ForceEffect;

/// `release_effect`'s Rust version.
pub fn release_effect(effect: u32) -> Result<(), BadInput> {
    let object = at(effect, field!(ForceEffect, object))?;
    if object != 0 {
        crate::input::di_objects::stop(object);
        crate::input::di_objects::release(object);
        put(effect, field!(ForceEffect, object), 0)?;
    }
    Ok(())
}

/// `start_effect`'s Rust version.
pub fn start_effect(effect: u32) -> Result<(), BadInput> {
    let object = at(effect, field!(ForceEffect, object))?;
    if object != 0 {
        crate::input::di_objects::download(object);
        crate::input::di_objects::start(object, 1, 0);
    }
    Ok(())
}

/// `set_effect_parameters`'s Rust version.
pub fn set_effect_parameters(effect: u32, flags: u32) -> Result<(), BadInput> {
    let object = at(effect, field!(ForceEffect, object))?;
    if object != 0 {
        crate::input::di_objects::set_parameters(
            object,
            effect.wrapping_add(field!(ForceEffect, effect)),
            flags,
        );
    }
    Ok(())
}

/// `stop_effect`'s Rust version.
pub fn stop_effect(effect: u32) -> Result<(), BadInput> {
    let object = at(effect, field!(ForceEffect, object))?;
    if object != 0 {
        crate::input::di_objects::stop(object);
    }
    Ok(())
}
