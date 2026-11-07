//! `effects::emit_particle_13` (0x4CE05E): one ring entry of kind 0xD for a car each frame,
//! at its model's point (`flt_769312` .., 12 bytes by the car's +4).
//!
//! In:  esi the car object
//! Out: every register kept (`pusha` .. `popa`)
//!
//! The entry: its place the point (x negated; y less `flt_7651E4`; by `flt_7651D4`) turned
//! by `g_marksCar`'s matrix (placed and the place taken off again, on the x87 stack:
//! `x87::turned_place_less`), less `flt_93A462` in y at +0x28; the strength
//! (`dword_9356C0`, a float) 1 - 1 / (the car's +8 by `flt_93A466` plus 1); its motion the
//! matrix's third row by the strength and `flt_93A46A` ..; then its place moved by the
//! motion times a random's low word by `flt_765214` times `g_marksFrameTicks`; a random
//! spin and turn (+0x54, +0x50); +0x58 0x3F; a life of 250 (+0x78 the strength by
//! `flt_7651A4` over it); +0x74 the strength by `dword_7651A0`; +0x68 `g_marksCar` and its
//! +0x88; +0x70 `flt_93A47E` by the strength plus `flt_93A482`, rounded, and +0x60 that by
//! `flt_93A486`. Caller: `sub_4C9AF6` (the cars' loop). Refuses (before anything) a null
//! object, a null `g_marksCar` or matrix, and a model (+4) past the table's 34 entries,
//! when an entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::paths::Paths;
use crate::effects::ring_entries::field;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::MarkCar;
use crate::session::{Dword, Float, read_first};

const MODELS: u32 = 34;
const LOOK: u32 = 0x3F;
const LIFE: u32 = 0xFA;

/// Why the Rust version refuses, if it does, worked out without writing.
fn refusal(ring: &Ring, object: &'static MarkCar) -> Result<Option<&'static str>, Refused> {
    Ok({
        // The ring full: nothing more is read.
        next_entry(ring)?;
        if !car_with_matrix()? {
            return Ok(Some("a null g_marksCar or matrix"));
        }
        if object.word(4)? >= MODELS {
            return Ok(Some("a car model past flt_769312's 34 entries"));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_particle_13(
    paths: &Paths,
    particles: &Particles,
    ring: &Ring,
    object: &'static MarkCar,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if let Some(why) = refusal(ring, object)? {
        return Err(BadInput(why).into());
    }
    if next_entry(ring)?.is_none() {
        return Ok(());
    }
    let model = object.word(4)? as usize;
    let e = take_entry(ring, effects, object.word(car_word::TAG)? & 0xFF_FFFF, 0xD)?;
    let put = |o: u32, v: u32| crate::effects::ring_entries::put(ring, e, o, v);
    let car = emitting_car(effects)?;
    let cells = car.whole()?;
    let t: [u32; 3] = read_first(&paths.spline_head.points[3 * model..]);
    let [x, y, y_less, z] = x87::turned_place_less(
        t,
        effects.setup.floats.plus_65_536.bits(),
        effects.setup.floats.seven_hundred.bits(),
        particles.kind13_set.lift.bits(),
        car.whole()?,
    );
    put(field::PLACE_X, x)?;
    put(field::PLACE_Y, y)?;
    put(field::GROUND_Y, y_less)?;
    put(field::PLACE_Z, z)?;
    let one: u32 = effects.setup.floats.one.bits();
    let s = x87::one_less_inverse(object.word(8)?, particles.kind13_set.strength_k.bits(), one);
    ring.work.whole.set(s);
    let motion: [u32; 3] = crate::effects::try_from_fn(|k| {
        Ok({
            x87::mul2(
                matrix_cell(cells, 0x20 + 4 * k as u32)?,
                s,
                particles.kind13_set.motion_k[k].bits(),
            )
        })
    })?;
    put(field::MOTION_X, motion[0])?;
    put(field::MOTION_Y, motion[1])?;
    put(field::MOTION_Z, motion[2])?;
    let r = (next_c5(&particles.kind13_set.drift_random) & 0xFFFF) as i32;
    let moved = x87::drifted(
        r,
        effects.setup.floats.over_65536.bits(),
        motion,
        effects.setup.clock.frame_ticks.get() as i32,
        [x, y, z],
    );
    put(field::PLACE_X, moved[0])?;
    put(field::PLACE_Y, moved[1])?;
    put(field::PLACE_Z, moved[2])?;
    let r = next_c5(&particles.kind13_set.spin_random);
    put(field::ANGLE_RATE, (r & 0x3F).wrapping_sub(0x20))?;
    put(field::ANGLE, r.rotate_right(6) & 0xFFF)?;
    put(field::CELL, LOOK)?;
    put(field::LIFE, LIFE)?;
    put(
        field::ALPHA_FADE,
        x87::times_over_int(s, effects.setup.floats.two_fifty_seven.bits(), LIFE as i32),
    )?;
    put(
        field::ALPHA,
        x87::mul(s, effects.setup.floats.two_fifty_six.bits()),
    )?;
    put(field::CAR, effects.kinds.emitter.car.get())?;
    let size = x87::times_plus_to_int(
        particles.kind13_set.size_k[0].bits(),
        s,
        particles.kind13_set.size_k[1].bits(),
    );
    put(field::SIZE_GOAL, size as u32)?;
    put(field::CAR_PACE, car.word(car_word::PACE)?)?;
    put(
        field::SIZE,
        x87::int_times(size, particles.kind13_set.size_k[2].bits()),
    )?;
    Ok(())
}

/// Kind 0xD's emitter variables (`emit_particle_13`) (IDA 0x93A462 .. 0x93A48A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Kind13Set {
    /// 0x93A462: `flt_93A462`: the place's lift
    pub lift: Float,
    /// 0x93A466: `flt_93A466`: the strength's factor
    pub strength_k: Float,
    /// 0x93A46A: `flt_93A46A` .. `flt_93A472`: the motion's factors (x, y, z)
    pub motion_k: [Float; 3],
    /// 0x93A476: `dword_93A476`: the drift's random
    pub drift_random: Dword,
    /// 0x93A47A: `dword_93A47A`: the spin's random
    pub spin_random: Dword,
    /// 0x93A47E: `flt_93A47E`, `flt_93A482`, `flt_93A486`: the size's factor, base and scale
    pub size_k: [Float; 3],
}

impl Kind13Set {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        Kind13Set {
            lift: Float::from_bits(0x47800000),
            strength_k: Float::from_bits(0x38D1B717),
            motion_k: [
                Float::from_bits(0xC5000000),
                Float::from_bits(0xC5000000),
                Float::from_bits(0xC5000000),
            ],
            drift_random: Dword::new(0x0),
            spin_random: Dword::new(0x0),
            size_k: [
                Float::from_bits(0x47800000),
                Float::from_bits(0x45000000),
                Float::from_bits(0x3E4CCCCD),
            ],
        }
    }
}

impl Kind13Set {
    pub const START: usize = 0x93A462;
    pub const LEN: usize = 0x28;
}

#[cfg(test)]
mod kind13_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind13_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind13Set>(), Kind13Set::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.kind13_set)
                == crate::moved::image_bytes(Kind13Set::START, Kind13Set::LEN),
            "KIND13_SET starts differently from the image"
        );
        assert_eq!(offset_of!(Kind13Set, lift), 0x93A462 - Kind13Set::START);
        assert_eq!(
            offset_of!(Kind13Set, strength_k),
            0x93A466 - Kind13Set::START
        );
        assert_eq!(offset_of!(Kind13Set, motion_k), 0x93A46A - Kind13Set::START);
        assert_eq!(
            offset_of!(Kind13Set, drift_random),
            0x93A476 - Kind13Set::START
        );
        assert_eq!(
            offset_of!(Kind13Set, spin_random),
            0x93A47A - Kind13Set::START
        );
        assert_eq!(offset_of!(Kind13Set, size_k), 0x93A47E - Kind13Set::START);
    }
}
