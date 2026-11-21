//! `effects::emit_twin_particle_second`: `effects::emit_twin_particle_first`'s code on its
//! own variables: kind 0x16, the level the car's `levels[1]`, the flags 0x33,
//! `TwinTuning.chance_random`, `TwinTuning.lift` .. `TwinTuning.motion_k[2]`,
//! `TwinTuning.spin_random`.
//!
//! Refuses as `emit_twin_particle_first`.

use crate::effects::emit_twin_particle_first::{TwinSet, emit_twin};
use crate::effects::emit_twin_particle_first_store::TwinTuning;
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::effects_settings_store::CarEffects;

/// `emit_twin_particle_second`'s variables.
pub(super) fn twenty_second(effects: &Effects) -> TwinSet<'_> {
    TwinSet {
        kind: 0x16,
        level: 1,
        flags: 0x33,
        tuning: &effects.particles.twenty_second_tuning,
    }
}

pub(super) fn emit_twin_particle_second(
    effects: &Effects,
    object: &CarEffects,
) -> Result<(), Refused> {
    emit_twin(effects, &twenty_second(effects), object)
}

impl TwinTuning {
    /// The values the game starts with.
    pub(super) const fn twenty_second() -> Self {
        TwinTuning {
            chance_random: 0x0,
            lift: 65536.0,
            row_k: [128.0, 128.0, 128.0],
            spread: 64.0,
            motion_k: [0.8, 0.8, 0.8],
            spin_random: 0x0,
        }
    }
}
