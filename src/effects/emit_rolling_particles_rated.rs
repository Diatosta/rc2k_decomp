//! `effects::emit_rolling_particles_rated` (0x4CCA58, a chunk of `effects::emit_wheel_particles` entered
//! from the kind 1 chunk when the kind-1 choice is not 0): one ring entry of kind 0x15 for the
//! wheel kind 1, when the rate gate passes.
//!
//! Takes: the wheel record, the car object
//!
//! The rate (`rate` in the ring's work): (1 - 1 / (the wheel's +0xC by the set's first two
//! factors, plus its third)) by its fourth; then the gate of `effects/emitters.rs`. The entry:
//! its place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the car matrix's first
//! row, 0 or 1 times its third), less the set's `lift` in y at +0x28; its motion per axis the
//! matrix's first row by the wheel's +0x10, its third by +0x18 and by +0xC, each by a
//! factor (`motion_k`); a random spin and turn (+0x54, +0x50); +0x58 0x34; a life of 1000
//! (+0x78 a constant over it); +0x74 the fade constant; the car (+0x68) and its +0x88, +0x90
//! plus the rate; +0x60 that sum by `size_k` (after a first value by the effects' random,
//! which still steps). Refuses (before anything) a null wheel, a null object, and a null car
//! or matrix when an entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::scene::matrix3::rounded_value;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

const LOOK: u32 = 0x34;
const LIFE: u32 = 0x3E8;

/// The rate, from the wheel's +0xC.
fn rate(
    particles: &Particles,
    wheel: &MarkRecord,
    effects: &crate::effects::owner::Effects,
) -> Result<i32, Refused> {
    Ok({
        let [k1, k2, k3, k4] = particles
            .kind21_set
            .rate_k
            .each_ref()
            .map(super::super::session::PlainFloat::get);
        x87::speed_rate_to_int(
            &wheel.driven_speed,
            k1,
            k2,
            k3,
            &effects.setup.floats.one,
            k4,
        )
    })
}

/// Why the emitter refuses, if it does, worked out without writing.
fn refusal(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &'static CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !gate_with(wheel, object, rate(particles, wheel, effects)?)?.passes {
            return Ok(None);
        }
        // The ring full: nothing more is read.
        next_entry(ring)?;
        if !car_with_matrix(effects)? {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

pub fn emit_rolling_particles_rated(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &'static CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(particles, ring, wheel, object, effects)? {
            return Err(BadInput(why).into());
        }
        let g = gate_with(wheel, object, rate(particles, wheel, effects)?)?;
        ring.work.rate.set(g.rate as u32);
        if let Some(count) = g.count {
            wheel.pace.set(count);
        }
        if !g.passes || next_entry(ring)?.is_none() {
            return Ok(());
        }
        let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, 0x15)?;
        let slot = crate::effects::ring_entries::Slot::new(ring, e);
        let car = emitting_car(effects)?;
        let m = car.whole(effects.car_blocks()?)?;
        let r = next_c5(&particles.kind21_set.side_random);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        effects.kinds.emitter.sides[0].set(a as u32);
        effects.kinds.emitter.sides[1].set(b as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            let k = 4 * k as u32;
            Ok(x87::pair_sum_80(
                a,
                matrix_float(m, k)?,
                b,
                matrix_float(m, 0x20 + k)?,
                &wheel.place[k as usize / 4],
            ))
        })?;
        slot.put(|en| &mut en.place[2], rounded_value(place[2]))?;
        slot.put(|en| &mut en.place[1], rounded_value(place[1]))?;
        slot.put(
            |en| &mut en.ground_y,
            x87::ext_less(place[1], &particles.kind21_set.lift),
        )?;
        slot.put(|en| &mut en.place[0], rounded_value(place[0]))?;
        let motion: [f32; 3] = crate::effects::try_from_fn(|k| {
            let (first, third) = (
                matrix_float(m, 4 * k as u32)?,
                matrix_float(m, 0x20 + 4 * k as u32)?,
            );
            let f = [0, 1, 2].map(|i| particles.kind21_set.motion_k[3 * k + i].get());
            Ok(x87::three_products([
                [first, wheel.local_velocity_x.get(), f[0]],
                [third, wheel.local_velocity_z.get(), f[1]],
                [third, wheel.driven_speed.get(), f[2]],
            ]))
        })?;
        slot.put(|en| &mut en.motion[2], motion[2])?;
        slot.put(|en| &mut en.motion[1], motion[1])?;
        slot.put(|en| &mut en.motion[0], motion[0])?;
        let r = next_c5(&particles.kind21_set.spin_random);
        slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
        slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
        slot.put(|en| &mut en.cell, LOOK)?;
        let s = effects
            .setup
            .work
            .seed
            .get()
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        effects.setup.work.seed.set(s);
        slot.put(
            |en| &mut en.size,
            x87::from_int(((s & 0x3FFF) + 0x4000) as i32).to_bits(),
        )?;
        slot.put(|en| &mut en.life, LIFE)?;
        slot.put(
            |en| &mut en.alpha_fade,
            x87::over_int(&effects.setup.floats.two_fifty_seven, LIFE as i32),
        )?;
        slot.put(
            |en| &mut en.alpha,
            effects.setup.floats.two_fifty_five.bits(),
        )?;
        slot.put(|en| &mut en.car, effects.kinds.emitter.car.get())?;
        let size = car.rate.get().wrapping_add(g.rate as u32);
        slot.put(|en| &mut en.size_goal, size)?;
        slot.put(|en| &mut en.car_pace, car.pace.get())?;
        slot.put(
            |en| &mut en.size,
            x87::int_times(size as i32, &particles.kind21_set.size_k).to_bits(),
        )?;
    }
    Ok(())
}

/// Kind 0x15's emitter variables (`emit_rolling_particles_rated`) (IDA 0x93A312 .. 0x93A39A).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RollingRatedSet {
    /// 0x93A312: `flt_93A312` .. `flt_93A31E`: the rate's factors
    pub rate_k: [PlainFloat; 4],
    /// 0x93A322: `dword_93A322`: the side's random
    pub side_random: PlainDword,
    /// 0x93A326: `flt_93A326`: the place's drop in y
    pub lift: PlainFloat,
    /// 0x93A32A: `flt_93A32A` .. `flt_93A34A`: per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [PlainFloat; 9],
    /// 0x93A34E: `dword_93A34E`: the spin's random
    pub spin_random: PlainDword,
    /// 0x93A352: `flt_93A352`: the size's factor
    pub size_k: PlainFloat,
    /// 0x93A356: `flt_93A35E` .. `flt_93A396`: read only by code in no function
    pub _gap_93a356: PlainBytes<68>,
}

impl RollingRatedSet {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        RollingRatedSet {
            rate_k: [
                PlainFloat::new(65.336),
                PlainFloat::new(0.00002),
                PlainFloat::new(1.0),
                PlainFloat::new(196608.0),
            ],
            side_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            motion_k: [
                PlainFloat::new(49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(-49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(-49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(-49.152),
            ],
            spin_random: PlainDword::new(0x0),
            size_k: PlainFloat::new(0.2),
            _gap_93a356: PlainBytes::new([
                0x0A, 0xD7, 0x23, 0x45, 0x17, 0xB7, 0x51, 0x39, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00,
                0x40, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x47, 0x6F, 0x12, 0x83, 0x42,
                0xA6, 0x9B, 0x44, 0x42, 0x4C, 0x37, 0x99, 0xC1, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B,
                0x44, 0x42, 0x4C, 0x37, 0x99, 0xC1, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B, 0x44, 0x42,
                0x4C, 0x37, 0x99, 0xC1, 0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C, 0x3E,
            ]),
        }
    }
}

impl RollingRatedSet {
    pub const START: usize = 0x93A312;
    pub const LEN: usize = 0x88;
}

#[cfg(test)]
mod kind21_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind21_set_starts_as_the_games() {
        assert_eq!(size_of::<RollingRatedSet>(), RollingRatedSet::LEN);
        assert!(
            bytes(&crate::effects::owner::effects().particles.kind21_set)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.particles.kind21_set/136"
                ),
            "KIND21_SET starts differently from the image"
        );
        assert_eq!(offset_of!(RollingRatedSet, rate_k), 0x0);
        assert_eq!(offset_of!(RollingRatedSet, side_random), 0x10);
        assert_eq!(offset_of!(RollingRatedSet, lift), 0x14);
        assert_eq!(offset_of!(RollingRatedSet, motion_k), 0x18);
        assert_eq!(offset_of!(RollingRatedSet, spin_random), 0x3c);
        assert_eq!(offset_of!(RollingRatedSet, size_k), 0x40);
    }
}
