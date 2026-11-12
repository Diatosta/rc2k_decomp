//! `effects::emit_twin_particle_first` (0x4CE283): by a chance one ring entry of kind 0x13 for a
//! car each frame; `effects::emit_twin_particle_second` is the same code on its own
//! variables (`TwinSet`).
//!
//! Takes: the car object
//!
//! The level (the twin set's `level`) is the car's +0x4C0 (+0x4C4); a random's bits 16 .. 18
//! must be under it (signed). The wheel flags get 0x3E (0x33); the set's copy of the car's
//! +0x4C .. +0x54 is kept. The entry: its place (the set's start point) turned and placed by
//! the emitting car's matrix (`x87::turned_place_f32`), less the set's lift in y at +0x28; its
//! motion the matrix's second row by the set's factors, moved by `effects::jitter_point`
//! (the set's spread) plus the kept car values by two more factors; a random spin and turn;
//! the flags' low 6 bits; a size 0x4000 or 0x8000 by the random's bit 6 (the strength, +0x60
//! as a float); a life from the lives table by the level (+0x78 a constant over it); +0x74
//! the fade constant; +0x68 the emitting car. Caller: `effects::cars_particles` (the cars'
//! loop). Refuses (before anything) a null object, and when an entry is made a null car or
//! matrix and a level past the lives table's 9 entries (the original reads on).

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::CarEffects;
use crate::scene::matrix3::{extended, rounded_value};
use crate::session::{PlainBytes, PlainDword, PlainFloat};

const LEVELS: u32 = 9;

/// One twin's variables.
pub struct TwinSet<'a> {
    pub kind: u32,
    /// The index of the car's level in its `levels`.
    pub level: usize,
    pub flags: u32,
    pub tuning: &'a TwinTuning,
}

/// `emit_twin_particle_first`'s.
pub fn nineteenth(effects: &Effects) -> TwinSet<'_> {
    TwinSet {
        kind: 0x13,
        level: 0,
        flags: 0x3E,
        tuning: &effects.particles.nineteenth_tuning,
    }
}

/// Whether an entry is to be made (the chance and a free entry), worked out without
/// writing.
fn makes(ring: &Ring, set: &TwinSet<'_>, object: &'static CarEffects) -> Result<bool, Refused> {
    Ok({
        let level = object.levels[set.level].get() as i32;
        let r = set
            .tuning
            .chance_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        ((r.rotate_right(16) & 7) as i32) < level && next_entry(ring)?.is_some()
    })
}

/// Why the emitter refuses, if it does, worked out without writing.
pub fn refusal(
    effects: &Effects,
    ring: &Ring,
    set: &TwinSet<'_>,
    object: &'static CarEffects,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !makes(ring, set, object)? {
            return Ok(None);
        }
        if !car_with_matrix(effects)? {
            return Ok(Some("a null g_marksCar or matrix"));
        }
        if object.levels[set.level].get() >= LEVELS {
            return Ok(Some("a level past dword_939B86's 9 entries"));
        }
        None
    })
}

pub fn emit_twin_particle_first(
    effects: &Effects,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_twin(effects, &nineteenth(effects), object)
}

/// The shared body, on `set`'s variables.
pub fn emit_twin(
    effects: &Effects,
    set: &TwinSet<'_>,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    let (masks, particles, ring) = (&effects.masks, &effects.particles, &effects.ring);
    if let Some(why) = refusal(effects, ring, set, object)? {
        return Err(BadInput(why).into());
    }
    let level = object.levels[set.level].get();
    particles.twin_state.level.set(level);
    let r = next_c5(&set.tuning.chance_random);
    if (r.rotate_right(16) & 7) as i32 >= level as i32 {
        return Ok(());
    }
    masks.surface.wheel_flags.set(set.flags);
    let motion: [u32; 3] = crate::session::read_bits(&object.motion);
    crate::session::write_bits(&particles.twin_state.motion, motion);
    if next_entry(ring)?.is_none() {
        return Ok(());
    }
    let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, set.kind)?;
    let slot = crate::effects::ring_entries::Slot::new(ring, e);
    let car = emitting_car(effects)?;
    let cells = car.whole(effects.car_blocks()?)?;
    let [x, y, y_less, z] = x87::turned_place_f32(
        particles
            .twin_state
            .point
            .each_ref()
            .map(crate::session::PlainFloat::get),
        &set.tuning.lift,
        car.whole(effects.car_blocks()?)?,
    );
    slot.put(|en| &mut en.place[0], x)?;
    slot.put(|en| &mut en.place[1], y)?;
    slot.put(|en| &mut en.ground_y, y_less)?;
    slot.put(|en| &mut en.place[2], z)?;
    let row =
        crate::effects::try_from_fn::<u32, 3>(|i| matrix_cell(cells, [0x18u32, 0x14, 0x10][i]))?
            .into_iter()
            .zip(set.tuning.row_k.iter())
            .map(|(a, k)| x87::mul_80(a, k))
            .collect::<Vec<_>>();
    let (moved, _) = jitter_point(
        [extended(set.tuning.spread.get()), row[2], row[1], row[0]],
        effects,
    );
    let step = effects.setup.clock.reciprocal.get();
    for n in 0..3 {
        slot.put(
            |en| &mut en.motion[n],
            x87::product_plus(
                motion[n],
                step,
                &set.tuning.motion_k[n],
                rounded_value(moved[n]),
            ),
        )?;
    }
    let r = next_c5(&set.tuning.spin_random);
    slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
    slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
    slot.put(|en| &mut en.cell, set.flags & 0x3F)?;
    let size = (r.rotate_right(0x14) & 0x4000) + 0x4000;
    ring.work.whole.set(size);
    slot.put(|en| &mut en.size, x87::from_int(size as i32).to_bits())?;
    let life: u32 = masks.surface.lives_by_level[level as usize].get();
    slot.put(|en| &mut en.life, life)?;
    slot.put(
        |en| &mut en.alpha_fade,
        x87::over_int(&effects.setup.floats.two_fifty_seven, life as i32),
    )?;
    slot.put(
        |en| &mut en.alpha,
        effects.setup.floats.two_fifty_five.bits(),
    )?;
    slot.put(|en| &mut en.car, effects.kinds.emitter.car.get())?;
    Ok(())
}

/// The twin car emitters' shared work (`emit_twin_particle_first`, `emit_twin_particle_second`) (IDA 0x93A1B6 .. 0x93A1D6).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TwinState {
    /// 0x93A1B6: `flt_93A1B6` .. `flt_93A1BE`: the point the entry's place starts at (turned and placed by the car's matrix)
    pub point: [PlainFloat; 3],
    /// 0x93A1C2: `dword_93A1C2` .. `dword_93A1CA`: the car's +0x4C .. +0x54, kept
    pub motion: [PlainDword; 3],
    /// 0x93A1CE: `dword_93A1CE`: the car's level (its +0x4C0 or +0x4C4)
    pub level: PlainDword,
    /// 0x93A1D2: Not named: 4 bytes (a gap, or unreferenced data)
    pub _gap_93a1d2: PlainBytes<4>,
}

impl TwinState {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        TwinState {
            point: [
                PlainFloat::new(0.0),
                PlainFloat::new(8192.0),
                PlainFloat::new(101072.0),
            ],
            motion: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            level: PlainDword::new(0x0),
            _gap_93a1d2: PlainBytes::new([0x0A, 0xD7, 0xA3, 0x3B]),
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
            bytes(&crate::effects::owner::effects().particles.twin_state)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.particles.twin_state/32"
                ),
            "TWIN_STATE starts differently from the image"
        );
        assert_eq!(offset_of!(TwinState, point), 0x0);
        assert_eq!(offset_of!(TwinState, motion), 0xc);
        assert_eq!(offset_of!(TwinState, level), 0x18);
    }
}

/// One twin car emitter's variables (`emit_twin_particle_first`'s at 0x93A48A, `emit_twin_particle_second`'s at 0x93A4B2) (IDA 0x93A48A .. 0x93A4B2).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TwinTuning {
    /// 0x93A48A: the chance's random
    pub chance_random: PlainDword,
    /// 0x93A48E: the place's lift
    pub lift: PlainFloat,
    /// 0x93A492: the second row's factors (z, y, x)
    pub row_k: [PlainFloat; 3],
    /// 0x93A49E: the spread `jitter_point` takes
    pub spread: PlainFloat,
    /// 0x93A4A2: the motion's factors (x, y, z)
    pub motion_k: [PlainFloat; 3],
    /// 0x93A4AE: the spin's random
    pub spin_random: PlainDword,
}

impl TwinTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn nineteenth() -> Self {
        TwinTuning {
            chance_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            row_k: [
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
            ],
            spread: PlainFloat::new(128.0),
            motion_k: [
                PlainFloat::new(0.8),
                PlainFloat::new(0.8),
                PlainFloat::new(0.8),
            ],
            spin_random: PlainDword::new(0x0),
        }
    }
}

impl TwinTuning {
    pub const LEN: usize = 0x28;
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
            bytes(&crate::effects::owner::effects().particles.nineteenth_tuning)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.particles.nineteenth_tuning/40"
                ),
            "NINETEENTH_TUNING starts differently from the image"
        );
        assert_eq!(offset_of!(TwinTuning, chance_random), 0x0);
        assert_eq!(offset_of!(TwinTuning, lift), 0x4);
        assert_eq!(offset_of!(TwinTuning, row_k), 0x8);
        assert_eq!(offset_of!(TwinTuning, spread), 0x14);
        assert_eq!(offset_of!(TwinTuning, motion_k), 0x18);
        assert_eq!(offset_of!(TwinTuning, spin_random), 0x24);
    }
}
