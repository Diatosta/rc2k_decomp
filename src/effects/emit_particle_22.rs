//! `effects::emit_particle_22` (0x4CE482): `effects::emit_particle_19`'s code on its own
//! variables: kind 0x16, the level the car's +0x4C4, the flags 0x33, `dword_93A4B2`,
//! `flt_93A4B6` .. `flt_93A4D2`, `dword_93A4D6` (an instruction diff: nothing else
//! differs).
//!
//! In:  esi the car object
//! Out: every register kept
//!
//! Refuses as `emit_particle_19`.

use crate::effects::emit_particle_19::TwinTuning;
use crate::effects::emit_particle_19::{TwinSet, emit_twin};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::Refused;
use crate::race::marks_settings_store::MarkCar;
use crate::session::{Dword, Float};

/// `emit_particle_22`'s variables.
pub static TWENTY_SECOND: TwinSet = TwinSet {
    kind: 0x16,
    level_at: 0x4C4,
    flags: 0x33,
    tuning: &crate::effects::owner::EFFECTS
        .particles
        .twenty_second_tuning,
};

/// The Rust version (see the header).
pub fn emit_particle_22(
    masks: &Masks,
    particles: &Particles,
    ring: &Ring,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    emit_twin(masks, particles, ring, &TWENTY_SECOND, object)
}

impl TwinTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn twenty_second() -> Self {
        TwinTuning {
            chance_random: Dword::new(0x0),
            lift: Float::from_bits(0x47800000),
            row_k: [
                Float::from_bits(0x43000000),
                Float::from_bits(0x43000000),
                Float::from_bits(0x43000000),
            ],
            spread: Float::from_bits(0x42800000),
            motion_k: [
                Float::from_bits(0x3F4CCCCD),
                Float::from_bits(0x3F4CCCCD),
                Float::from_bits(0x3F4CCCCD),
            ],
            spin_random: Dword::new(0x0),
        }
    }
}

/// Marker for `moved::MOVED`: TWENTY_SECOND_TUNING's place in the image.
pub struct TwentySecondTuning;

impl TwentySecondTuning {
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
            ) == crate::moved::image_bytes(TwentySecondTuning::START, TwinTuning::LEN),
            "TWENTY_SECOND_TUNING starts differently from the image"
        );
    }
}
