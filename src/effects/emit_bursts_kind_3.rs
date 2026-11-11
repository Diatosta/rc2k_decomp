//! `effects::emit_bursts_kind_3` (0x4CD747): `effects::emit_bursts` on its own factors and randoms
//! (`BurstTuning.strength_k`, `BurstTuning.blend_k[0]`, `BurstTuning.blend_k[1]`, `BurstTuning.first[0]` .. `BurstTuning.second[2]`); the code
//! is the same instruction for instruction, the ring-full slip included (0x4CDC6E's
//! exit; section 16).
//!
//! Takes: the wheel record, the car object
//!
//! See `marks/emit_bursts.rs`; no variable of its own shared but the entries' and the
//! scratch ones. Caller: `effects::emit_wheel_particles`' kind 3. Refuses as `emit_bursts`.

use crate::effects::emit_bursts::BurstTuning;
use crate::effects::emit_bursts::{BurstSet, HALVES, emit_bursts_with};
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::session::{PlainDword, PlainFloat};

/// `emit_bursts_kind_3`'s: `emit_bursts`' with its own factors and randoms.
pub const SECOND: BurstSet = BurstSet {
    halves: HALVES,
    tuning: &crate::effects::owner::EFFECTS
        .unchecked()
        .particles
        .second_tuning,
};

pub fn emit_bursts_kind_3(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_bursts_with(effects, &SECOND, wheel, object)
}

impl BurstTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn second() -> Self {
        BurstTuning {
            strength_k: PlainFloat::new(-0.25),
            first: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            blend_k: [PlainFloat::new(-0.25), PlainFloat::new(-0.25)],
            second: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
        }
    }
}

/// Marker for `moved::MOVED`: SECOND_TUNING's place in the image.
pub struct BurstKind3Tuning;

impl BurstKind3Tuning {
    pub const START: usize = 0x93A3F6;
    pub const LEN: usize = BurstTuning::LEN;
}

#[cfg(test)]
mod second_tuning_tests {
    use super::*;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn second_tuning_starts_as_the_games() {
        assert_eq!(size_of::<BurstTuning>(), BurstTuning::LEN);
        assert!(
            bytes(&crate::effects::owner::effects().particles.second_tuning)
                == crate::moved::image_bytes(BurstKind3Tuning::START, BurstTuning::LEN),
            "SECOND_TUNING starts differently from the image"
        );
    }
}
