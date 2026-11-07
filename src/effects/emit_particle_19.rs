//! `effects::emit_particle_19` (0x4CE283): by a chance one ring entry of kind 0x13 for a
//! car each frame; `effects::emit_particle_22` (0x4CE482) is the same code on its own
//! variables (`TwinSet`).
//!
//! In:  esi the car object
//! Out: every register kept (`pusha` .. `popa`)
//!
//! The level (`dword_93A1CE`) the car's +0x4C0 (+0x4C4); a random's bits 16 .. 18 must be
//! under it (signed). `g_marksWheelFlags` 0x3E (0x33); `dword_93A1C2` .. the car's +0x4C ..
//! +0x54. The entry: its place `flt_93A1B6` .. turned and placed by `g_marksCar`'s matrix
//! (`x87::turned_place`), less `flt_93A48E` in y at +0x28; its motion the matrix's second
//! row by `flt_93A492` .. moved by `effects::jitter_point` (spread `flt_93A49E`) plus
//! `dword_93A1C2` .. by `flt_764FE0` and `flt_93A4A2` ..; a random spin and turn; the
//! flags' low 6 bits; a size 0x4000 or 0x8000 by the random's bit 6 (`dword_9356C0`, +0x60
//! as a float); a life `dword_939B86[level]` (+0x78 `flt_7651A4` over it); +0x74
//! `dword_76519C`; +0x68 `g_marksCar`. Caller: `sub_4C9AF6` (the cars' loop). Refuses
//! (before anything) a null object, and when an entry is made a null `g_marksCar` or matrix
//! and a level past `dword_939B86`'s 9 entries (the original reads on).

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::mask_store::Masks;
use crate::effects::owner::effects;
use crate::effects::ring_entries::field;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::MarkCar;
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Bytes, Dword, Float};

const LEVELS: u32 = 9;

/// One twin's variables.
pub struct TwinSet {
    pub kind: u32,
    /// The car's field holding the level.
    pub level_at: u32,
    pub flags: u32,
    pub tuning: &'static TwinTuning,
}

/// `emit_particle_19`'s.
pub static NINETEENTH: TwinSet = TwinSet {
    kind: 0x13,
    level_at: 0x4C0,
    flags: 0x3E,
    tuning: &crate::effects::owner::EFFECTS.particles.nineteenth_tuning,
};

/// Whether an entry is to be made (the chance and a free entry), worked out without
/// writing.
fn makes(ring: &Ring, set: &TwinSet, object: &'static MarkCar) -> Result<bool, Refused> {
    Ok({
        let level = object.word(set.level_at)? as i32;
        let r = set
            .tuning
            .chance_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        ((r.rotate_right(16) & 7) as i32) < level && next_entry(ring)?.is_some()
    })
}

/// Why the Rust version refuses, if it does, worked out without writing.
pub fn refusal(
    ring: &Ring,
    set: &TwinSet,
    object: &'static MarkCar,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !makes(ring, set, object)? {
            return Ok(None);
        }
        if !car_with_matrix()? {
            return Ok(Some("a null g_marksCar or matrix"));
        }
        if object.word(set.level_at)? >= LEVELS {
            return Ok(Some("a level past dword_939B86's 9 entries"));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_particle_19(
    masks: &Masks,
    particles: &Particles,
    ring: &Ring,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    emit_twin(masks, particles, ring, &NINETEENTH, object)
}

/// The shared body, on `set`'s variables.
pub fn emit_twin(
    masks: &Masks,
    particles: &Particles,
    ring: &Ring,
    set: &TwinSet,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    let effects = effects();
    if let Some(why) = refusal(ring, set, object)? {
        return Err(BadInput(why).into());
    }
    let level = object.word(set.level_at)?;
    particles.twin_state.level.set(level);
    let r = next_c5(&set.tuning.chance_random);
    if (r.rotate_right(16) & 7) as i32 >= level as i32 {
        return Ok(());
    }
    masks.surface.wheel_flags.set(set.flags);
    let motion = [object.word(0x4C)?, object.word(0x50)?, object.word(0x54)?];
    crate::session::write_bits(&particles.twin_state.motion, motion);
    if next_entry(ring)?.is_none() {
        return Ok(());
    }
    let e = take_entry(ring, object.word(0x1C)? & 0xFF_FFFF, set.kind)?;
    let put = |o: u32, v: u32| crate::effects::ring_entries::put(ring, e, o, v);
    let car = emitting_car(effects)?;
    let cells = car.whole()?;
    let [x, y, y_less, z] = x87::turned_place(
        crate::session::read_bits(&particles.twin_state.point),
        set.tuning.lift.bits(),
        car.whole()?,
    );
    put(field::PLACE_X, x)?;
    put(field::PLACE_Y, y)?;
    put(field::GROUND_Y, y_less)?;
    put(field::PLACE_Z, z)?;
    let row =
        crate::effects::try_from_fn::<u32, 3>(|i| matrix_cell(cells, [0x18u32, 0x14, 0x10][i]))?
            .into_iter()
            .zip(set.tuning.row_k.iter())
            .map(|(a, k)| x87::mul_80(a, k.bits()))
            .collect::<Vec<_>>();
    let (moved, _) = jitter_point(
        [extended(set.tuning.spread.bits()), row[2], row[1], row[0]],
        effects,
    );
    let step: u32 = effects.setup.clock.reciprocal.bits();
    for n in 0..3 {
        put(
            field::MOTION_X + 4 * n as u32,
            x87::product_plus(
                motion[n],
                step,
                set.tuning.motion_k[n].bits(),
                rounded(moved[n]),
            ),
        )?;
    }
    let r = next_c5(&set.tuning.spin_random);
    put(field::ANGLE_RATE, (r & 0x3F).wrapping_sub(0x20))?;
    put(field::ANGLE, r.rotate_right(6) & 0xFFF)?;
    put(field::CELL, set.flags & 0x3F)?;
    let size = (r.rotate_right(0x14) & 0x4000) + 0x4000;
    ring.work.whole.set(size);
    put(field::SIZE, x87::from_int(size as i32))?;
    let life: u32 = masks.surface.lives_by_level[level as usize].get();
    put(field::LIFE, life)?;
    put(
        field::ALPHA_FADE,
        x87::over_int(effects.setup.floats.two_fifty_seven.bits(), life as i32),
    )?;
    put(field::ALPHA, effects.setup.floats.two_fifty_five.bits())?;
    put(field::CAR, effects.kinds.emitter.car.get())?;
    Ok(())
}

/// The twin car emitters' shared work (`emit_particle_19`, `emit_particle_22`) (IDA 0x93A1B6 .. 0x93A1D6, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TwinState {
    /// 0x93A1B6: `flt_93A1B6` .. `flt_93A1BE`: the point the entry's place starts at (turned and placed by the car's matrix)
    pub point: [Float; 3],
    /// 0x93A1C2: `dword_93A1C2` .. `dword_93A1CA`: the car's +0x4C .. +0x54, kept
    pub motion: [Dword; 3],
    /// 0x93A1CE: `dword_93A1CE`: the car's level (its +0x4C0 or +0x4C4)
    pub level: Dword,
    /// 0x93A1D2: Not named: 4 bytes (a gap, or unreferenced data)
    pub _gap_93a1d2: Bytes<4>,
}

impl TwinState {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        TwinState {
            point: [
                Float::from_bits(0x0),
                Float::from_bits(0x46000000),
                Float::from_bits(0x47C56800),
            ],
            motion: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
            level: Dword::new(0x0),
            _gap_93a1d2: Bytes::new([0x0A, 0xD7, 0xA3, 0x3B]),
        }
    }
}

impl TwinState {
    pub const START: usize = 0x93A1B6;
    pub const LEN: usize = 0x20;
}

#[cfg(test)]
mod twin_state_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn twin_state_starts_as_the_games() {
        assert_eq!(size_of::<TwinState>(), TwinState::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.twin_state)
                == crate::moved::image_bytes(TwinState::START, TwinState::LEN),
            "TWIN_STATE starts differently from the image"
        );
        assert_eq!(offset_of!(TwinState, point), 0x93A1B6 - TwinState::START);
        assert_eq!(offset_of!(TwinState, motion), 0x93A1C2 - TwinState::START);
        assert_eq!(offset_of!(TwinState, level), 0x93A1CE - TwinState::START);
    }
}

/// One twin car emitter's variables (`emit_particle_19`'s at 0x93A48A, `emit_particle_22`'s at 0x93A4B2) (IDA 0x93A48A .. 0x93A4B2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TwinTuning {
    /// 0x93A48A: the chance's random
    pub chance_random: Dword,
    /// 0x93A48E: the place's lift
    pub lift: Float,
    /// 0x93A492: the second row's factors (z, y, x)
    pub row_k: [Float; 3],
    /// 0x93A49E: the spread `jitter_point` takes
    pub spread: Float,
    /// 0x93A4A2: the motion's factors (x, y, z)
    pub motion_k: [Float; 3],
    /// 0x93A4AE: the spin's random
    pub spin_random: Dword,
}

impl TwinTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn nineteenth() -> Self {
        TwinTuning {
            chance_random: Dword::new(0x0),
            lift: Float::from_bits(0x47800000),
            row_k: [
                Float::from_bits(0x43800000),
                Float::from_bits(0x43800000),
                Float::from_bits(0x43800000),
            ],
            spread: Float::from_bits(0x43000000),
            motion_k: [
                Float::from_bits(0x3F4CCCCD),
                Float::from_bits(0x3F4CCCCD),
                Float::from_bits(0x3F4CCCCD),
            ],
            spin_random: Dword::new(0x0),
        }
    }
}

impl TwinTuning {
    pub const LEN: usize = 0x28;
}

/// Marker for `moved::MOVED`: NINETEENTH_TUNING's place in the image.
pub struct NineteenthTuning;

impl NineteenthTuning {
    pub const START: usize = 0x93A48A;
    pub const LEN: usize = TwinTuning::LEN;
}

#[cfg(test)]
mod nineteenth_tuning_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn nineteenth_tuning_starts_as_the_games() {
        assert_eq!(size_of::<TwinTuning>(), TwinTuning::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.nineteenth_tuning)
                == crate::moved::image_bytes(NineteenthTuning::START, TwinTuning::LEN),
            "NINETEENTH_TUNING starts differently from the image"
        );
        assert_eq!(
            offset_of!(TwinTuning, chance_random),
            0x93A48A - NineteenthTuning::START
        );
        assert_eq!(
            offset_of!(TwinTuning, lift),
            0x93A48E - NineteenthTuning::START
        );
        assert_eq!(
            offset_of!(TwinTuning, row_k),
            0x93A492 - NineteenthTuning::START
        );
        assert_eq!(
            offset_of!(TwinTuning, spread),
            0x93A49E - NineteenthTuning::START
        );
        assert_eq!(
            offset_of!(TwinTuning, motion_k),
            0x93A4A2 - NineteenthTuning::START
        );
        assert_eq!(
            offset_of!(TwinTuning, spin_random),
            0x93A4AE - NineteenthTuning::START
        );
    }
}
