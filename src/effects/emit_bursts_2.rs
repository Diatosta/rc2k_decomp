//! `effects::emit_bursts_2` (0x4CD747): `effects::emit_bursts` on its own factors and randoms
//! (`flt_93A3F6`, `flt_93A406`, `flt_93A40A`, `dword_93A3FA` .. `dword_93A416`); the code
//! is the same instruction for instruction, the ring-full slip included (0x4CDC6E's
//! exit; section 16).
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! See `marks/emit_bursts.rs`; no variable of its own shared but the entries' and the
//! scratch ones. Caller: `effects::emit_wheel_particles`' kind 3. Refuses as `emit_bursts`.

use crate::effects::emit_bursts::BurstTuning;
use crate::effects::emit_bursts::{BurstSet, HALVES, emit_bursts_with};
use crate::effects::ring_store::Ring;
use crate::errors::Refused;
use crate::race::marks_settings_store::{MarkCar, MarkRecord};
use crate::session::{Dword, Float};

/// `emit_bursts_2`'s: `emit_bursts`' with its own factors and randoms.
pub static SECOND: BurstSet = BurstSet {
    halves: HALVES,
    tuning: &crate::effects::owner::EFFECTS.particles.second_tuning,
};

/// The Rust version (see the header).
pub fn emit_bursts_2(
    ring: &Ring,
    wheel: &MarkRecord,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    emit_bursts_with(ring, &SECOND, wheel, object)
}

impl BurstTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn second() -> Self {
        BurstTuning {
            strength_k: Float::from_bits(0xBE800000),
            first: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
            blend_k: [Float::from_bits(0xBE800000), Float::from_bits(0xBE800000)],
            second: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
        }
    }
}

/// Marker for `moved::MOVED`: SECOND_TUNING's place in the image.
pub struct SecondTuning;

impl SecondTuning {
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
            bytes(&crate::effects::owner::EFFECTS.particles.second_tuning)
                == crate::moved::image_bytes(SecondTuning::START, BurstTuning::LEN),
            "SECOND_TUNING starts differently from the image"
        );
    }
}
