//! A force-feedback effect's DirectInput object (`IDirectInputEffect`, an effect's +0x30;
//! the effects are a device record's six from +0x3C, 0xA4 bytes each), asked by the game's
//! four small functions, each doing nothing when the effect has no object:
//! - `input::release_effect`: `Stop`, `Release`, +0x30 0;
//! - `input::start_effect`: `Download`, `Start` (one iteration, no flags);
//! - `input::set_effect_parameters`: `SetParameters` with the effect's
//!   `DIEFFECT` (+0x34) and the flags;
//! - `input::stop_effect`: `Stop`.

use crate::input::di_objects::EffectParameters;
use crate::input::types::ForceEffect;

/// What the DirectInput stand-in takes from the effect's `DIEFFECT`. Its pointers
/// (`directions`, `envelope`, `type_specific`) say whether the part is there; they always
/// point at the effect's own arrays (`effect_changes`, `make_effect`), so those are read.
pub(super) fn parameters_of(effect: &ForceEffect) -> EffectParameters {
    let d = &effect.effect;
    let specific = d.type_specific_size.get() / 4;
    EffectParameters {
        duration: d.duration.get(),
        gain: d.gain.get(),
        direction: (d.directions.is_set() && d.axes_count.get() > 0)
            .then(|| effect.directions[0].get() as i32),
        envelope: (d.envelope.is_set()).then(|| {
            let e = &effect.envelope;
            [
                e.attack_level.get(),
                e.attack_time.get(),
                e.fade_level.get(),
                e.fade_time.get(),
            ]
        }),
        type_specific: (d.type_specific.is_set()).then(|| {
            effect
                .type_specific
                .iter()
                .take(specific as usize)
                .map(|v| v.get() as i32)
                .collect()
        }),
    }
}

/// Gives the effect's DirectInput object back.
pub(super) fn release_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::stop(object);
        crate::input::di_objects::release(object);
        effect.object.set(0);
    }
}

/// Downloads and starts the effect.
pub(super) fn start_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::download(object);
        crate::input::di_objects::start(object, 1, 0);
    }
}

/// Sets the effect's parameters with `flags`.
pub(super) fn set_effect_parameters(effect: &ForceEffect, flags: u32) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::set_parameters(object, &parameters_of(effect), flags);
    }
}

/// Stops the effect.
pub fn stop_effect(effect: &ForceEffect) {
    let object = effect.object.get();
    if object != 0 {
        crate::input::di_objects::stop(object);
    }
}
