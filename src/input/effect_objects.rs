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

use crate::input::di_objects::EffectParameters;
use crate::input::types::ForceEffect;

/// What the DirectInput stand-in takes from the effect's `DIEFFECT`. Its pointers
/// (`directions`, `envelope`, `type_specific`) say whether the part is there; they always
/// point at the effect's own arrays (`effect_changes`, `make_effect`), so those are read.
pub fn parameters_of(effect: &ForceEffect) -> EffectParameters {
    let d = &effect.effect;
    let specific = d.type_specific_size.get() / 4;
    EffectParameters {
        duration: d.duration.get(),
        gain: d.gain.get(),
        direction: (d.directions.get() != 0 && d.axes_count.get() > 0)
            .then(|| effect.directions[0].get() as i32),
        envelope: (d.envelope.get() != 0).then(|| {
            let e = &effect.envelope;
            [
                e.attack_level.get(),
                e.attack_time.get(),
                e.fade_level.get(),
                e.fade_time.get(),
            ]
        }),
        type_specific: (d.type_specific.get() != 0).then(|| {
            effect
                .type_specific
                .iter()
                .take(specific as usize)
                .map(|v| v.get() as i32)
                .collect()
        }),
    }
}

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
        crate::input::di_objects::set_parameters(object, &parameters_of(effect), flags);
    }
}

/// `stop_effect`'s Rust version.
pub fn stop_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::stop(object);
    }
}
