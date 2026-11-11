//! `effects::emit_twin_particle_second` (0x4CE482): `effects::emit_twin_particle_first`'s code on its own
//! variables: kind 0x16, the level the car's +0x4C4, the flags 0x33, `dword_93A4B2`,
//! `flt_93A4B6` .. `flt_93A4D2`, `dword_93A4D6` (an instruction diff: nothing else
//! differs).
//!
//! Takes: the car object
//!
//! Refuses as `emit_twin_particle_first`.

use crate::effects::emit_twin_particle_first::TwinTuning;
use crate::effects::emit_twin_particle_first::{TwinSet, emit_twin};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::Refused;
use crate::race::effects_settings_store::CarEffects;
use crate::session::{Dword, Float};

/// `emit_twin_particle_second`'s variables.
pub static TWENTY_SECOND: TwinSet = TwinSet {
    kind: 0x16,
    level: 1,
    flags: 0x33,
    tuning: &crate::effects::owner::EFFECTS
        .particles
        .twenty_second_tuning,
};

pub fn emit_twin_particle_second(
    masks: &Masks,
    particles: &Particles,
    ring: &Ring,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_twin(masks, particles, ring, &TWENTY_SECOND, object)
}

impl TwinTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn twenty_second() -> Self {
        TwinTuning {
            chance_random: Dword::new(0x0),
            lift: Float::new(65536.0),
            row_k: [Float::new(128.0), Float::new(128.0), Float::new(128.0)],
            spread: Float::new(64.0),
            motion_k: [Float::new(0.8), Float::new(0.8), Float::new(0.8)],
            spin_random: Dword::new(0x0),
        }
    }
}

/// Marker for `moved::MOVED`: TWENTY_SECOND_TUNING's place in the image.
pub struct TwinSecondTuning;

impl TwinSecondTuning {
    pub const START: usize = 0x93A4B2;
    pub const LEN: usize = TwinTuning::LEN;
}

#[cfg(test)]
mod twenty_second_tuning_tests {
    use super::*;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn twenty_second_tuning_starts_as_the_games() {
        assert_eq!(size_of::<TwinTuning>(), TwinTuning::LEN);
        assert!(
            bytes(
                &crate::effects::owner::EFFECTS
                    .particles
                    .twenty_second_tuning
            ) == crate::moved::image_bytes(TwinSecondTuning::START, TwinTuning::LEN),
            "TWENTY_SECOND_TUNING starts differently from the image"
        );
    }
}
