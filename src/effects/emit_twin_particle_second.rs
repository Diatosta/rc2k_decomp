//! `effects::emit_twin_particle_second` (0x4CE482): `effects::emit_twin_particle_first`'s code on its own
//! variables: kind 0x16, the level the car's +0x4C4, the flags 0x33, `TwinTuning.chance_random`,
//! `TwinTuning.lift` .. `TwinTuning.motion_k[2]`, `TwinTuning.spin_random` (an instruction diff: nothing else
//! differs).
//!
//! Takes: the car object
//!
//! Refuses as `emit_twin_particle_first`.

use crate::effects::emit_twin_particle_first::TwinTuning;
use crate::effects::emit_twin_particle_first::{TwinSet, emit_twin};
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::effects_settings_store::CarEffects;
use crate::session::{PlainDword, PlainFloat};

/// `emit_twin_particle_second`'s variables.
pub fn twenty_second(effects: &Effects) -> TwinSet<'_> {
    TwinSet {
        kind: 0x16,
        level: 1,
        flags: 0x33,
        tuning: &effects.particles.twenty_second_tuning,
    }
}

pub fn emit_twin_particle_second(
    effects: &Effects,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_twin(effects, &twenty_second(effects), object)
}

impl TwinTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn twenty_second() -> Self {
        TwinTuning {
            chance_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            row_k: [
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
            ],
            spread: PlainFloat::new(64.0),
            motion_k: [
                PlainFloat::new(0.8),
                PlainFloat::new(0.8),
                PlainFloat::new(0.8),
            ],
            spin_random: PlainDword::new(0x0),
        }
    }
}
