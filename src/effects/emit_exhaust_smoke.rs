//! `effects::emit_exhaust_smoke` (0x4CE05E): one ring entry of kind 0xD for a car each frame,
//! at its model's point (the looks table, 12 bytes by the car's +4).
//!
//! Takes: the car object
//!
//! The entry: its place the point (x negated; y less a constant; by a constant) turned by the
//! emitting car's matrix (placed and the place taken off again, on the x87 stack:
//! `x87::turned_place_less`), less the set's lift in y at +0x28; the strength (a float)
//! 1 - 1 / (the car's +8 by the set's factor plus 1); its motion the matrix's third row by
//! the strength and the set's factors; then its place moved by the motion times a random's
//! low word by a constant times the frame's ticks; a random spin and turn (+0x54, +0x50);
//! +0x58 0x3F; a life of 250 (+0x78 the strength by a constant over it); +0x74 the strength by
//! the fade constant; +0x68 the emitting car and its +0x88; +0x70 the set's size factor by the
//! strength plus its base, rounded, and +0x60 that by its scale. Caller:
//! `effects::cars_particles` (the cars' loop). Refuses (before anything) a null object, a null
//! car or matrix, and a model (+4) past the table's 34 entries, when an entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::paths::Paths;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::effects_settings_store::CarEffects;
use crate::session::{Dword, Float, read_first};

const MODELS: u32 = 34;
const LOOK: u32 = 0x3F;
const LIFE: u32 = 0xFA;

/// Why the emitter refuses, if it does, worked out without writing.
fn refusal(ring: &Ring, object: &'static CarEffects) -> Result<Option<&'static str>, Refused> {
    Ok({
        // The ring full: nothing more is read.
        next_entry(ring)?;
        if !car_with_matrix()? {
            return Ok(Some("a null g_marksCar or matrix"));
        }
        if object.model.get() >= MODELS {
            return Ok(Some("a car model past flt_769312's 34 entries"));
        }
        None
    })
}

pub fn emit_exhaust_smoke(
    paths: &Paths,
    particles: &Particles,
    ring: &Ring,
    object: &'static CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if let Some(why) = refusal(ring, object)? {
        return Err(BadInput(why).into());
    }
    if next_entry(ring)?.is_none() {
        return Ok(());
    }
    let model = object.model.get() as usize;
    let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, 0xD)?;
    let slot = crate::effects::ring_entries::Slot::new(ring, e);
    let car = emitting_car(effects)?;
    let cells = car.whole()?;
    let t: [u32; 3] = read_first(&paths.spline_head.points[3 * model..]);
    let [x, y, y_less, z] = x87::turned_place_less(
        t,
        &effects.setup.floats.particle_13_place_k,
        &effects.setup.floats.seven_hundred,
        &particles.kind13_set.lift,
        car.whole()?,
    );
    slot.put(|en| &mut en.place[0], x)?;
    slot.put(|en| &mut en.place[1], y)?;
    slot.put(|en| &mut en.ground_y, y_less)?;
    slot.put(|en| &mut en.place[2], z)?;
    let one: u32 = effects.setup.floats.one.bits();
    let s = x87::one_less_inverse(&object.engine_speed, &particles.kind13_set.strength_k, one);
    ring.work.whole.set(s.to_bits());
    let motion: [u32; 3] = crate::effects::try_from_fn(|k| {
        Ok({
            x87::mul2(
                matrix_cell(cells, 0x20 + 4 * k as u32)?,
                s,
                &particles.kind13_set.motion_k[k],
            )
            .to_bits()
        })
    })?;
    slot.put(|en| &mut en.motion[0], motion[0])?;
    slot.put(|en| &mut en.motion[1], motion[1])?;
    slot.put(|en| &mut en.motion[2], motion[2])?;
    let r = (next_c5(&particles.kind13_set.drift_random) & 0xFFFF) as i32;
    let moved = x87::drifted(
        r,
        &effects.setup.floats.over_65536,
        motion,
        effects.setup.clock.frame_ticks.get() as i32,
        [x, y, z],
    );
    slot.put(|en| &mut en.place[0], moved[0].to_bits())?;
    slot.put(|en| &mut en.place[1], moved[1].to_bits())?;
    slot.put(|en| &mut en.place[2], moved[2].to_bits())?;
    let r = next_c5(&particles.kind13_set.spin_random);
    slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
    slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
    slot.put(|en| &mut en.cell, LOOK)?;
    slot.put(|en| &mut en.life, LIFE)?;
    slot.put(
        |en| &mut en.alpha_fade,
        bh::st(
            bh::ld(s.to_bits()) * bh::ld(effects.setup.floats.two_fifty_seven.bits())
                / bh::int(LIFE as i32),
        ),
    )?;
    slot.put(
        |en| &mut en.alpha,
        x87::mul(s, &effects.setup.floats.two_fifty_six).to_bits(),
    )?;
    slot.put(|en| &mut en.car, effects.kinds.emitter.car.get())?;
    let size = x87::times_plus_to_int(
        &particles.kind13_set.size_k[0],
        s,
        &particles.kind13_set.size_k[1],
    );
    slot.put(|en| &mut en.size_goal, size as u32)?;
    slot.put(|en| &mut en.car_pace, car.pace.get())?;
    slot.put(
        |en| &mut en.size,
        x87::int_times(size, &particles.kind13_set.size_k[2]).to_bits(),
    )?;
    Ok(())
}

/// Kind 0xD's emitter variables (`emit_exhaust_smoke`) (IDA 0x93A462 .. 0x93A48A).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ExhaustSet {
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

impl ExhaustSet {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        ExhaustSet {
            lift: Float::new(65536.0),
            strength_k: Float::new(0.0001),
            motion_k: [
                Float::new(-2048.0),
                Float::new(-2048.0),
                Float::new(-2048.0),
            ],
            drift_random: Dword::new(0x0),
            spin_random: Dword::new(0x0),
            size_k: [Float::new(65536.0), Float::new(2048.0), Float::new(0.2)],
        }
    }
}

impl ExhaustSet {
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
        assert_eq!(size_of::<ExhaustSet>(), ExhaustSet::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.kind13_set)
                == crate::moved::image_bytes(ExhaustSet::START, ExhaustSet::LEN),
            "KIND13_SET starts differently from the image"
        );
        assert_eq!(offset_of!(ExhaustSet, lift), 0x93A462 - ExhaustSet::START);
        assert_eq!(
            offset_of!(ExhaustSet, strength_k),
            0x93A466 - ExhaustSet::START
        );
        assert_eq!(
            offset_of!(ExhaustSet, motion_k),
            0x93A46A - ExhaustSet::START
        );
        assert_eq!(
            offset_of!(ExhaustSet, drift_random),
            0x93A476 - ExhaustSet::START
        );
        assert_eq!(
            offset_of!(ExhaustSet, spin_random),
            0x93A47A - ExhaustSet::START
        );
        assert_eq!(offset_of!(ExhaustSet, size_k), 0x93A47E - ExhaustSet::START);
    }
}
