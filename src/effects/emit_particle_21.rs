//! `effects::emit_particle_21` (0x4CCA58, a chunk of `effects::emit_wheel_particles` entered
//! by `jnz` from 0x4CCCDE when `dword_93A1D6` is not 0, left by `popa; retn`): one ring
//! entry of kind 0x15 for the wheel kind 1, when the rate gate passes.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The rate (`dword_9356B0`): (1 - 1 / (the wheel's +0xC by `flt_93A312`, `flt_93A316`,
//! plus `flt_93A31A`)) by `flt_93A31E`; then `marks/emitters.rs`' gate. The entry: its
//! place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the car matrix's first
//! row, 0 or 1 times its third), less `flt_93A326` in y at +0x28; its motion per axis the
//! matrix's first row by the wheel's +0x10, its third by +0x18 and by +0xC, each by a
//! factor (`flt_93A32A` ..); a random spin and turn (+0x54, +0x50); +0x58 0x34; a life of
//! 1000 (+0x78 `flt_7651A4` over it); +0x74 `dword_76519C`; the car (+0x68) and its +0x88,
//! +0x90 plus the rate; +0x60 that sum by `flt_93A352` (after a first value by the marks'
//! random, which still steps). Refuses (before anything) a null wheel, a null object, and a
//! null car or matrix when an entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::ring_entries::field;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::{MarkCar, MarkRecord};
use crate::scene::matrix3::rounded;
use crate::session::{Bytes, Dword, Float};

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
            .map(super::super::session::Float::bits);
        x87::speed_rate_to_int(
            wheel.speed_c.bits(),
            k1,
            k2,
            k3,
            effects.setup.floats.one.bits(),
            k4,
        )
    })
}

/// Why the Rust version refuses these registers, if it does, worked out without writing.
fn refusal(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &'static MarkCar,
    effects: &crate::effects::owner::Effects,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !gate_with(wheel, object, rate(particles, wheel, effects)?)?.passes {
            return Ok(None);
        }
        // The ring full: nothing more is read.
        next_entry(ring)?;
        if !car_with_matrix()? {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_particle_21(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &'static MarkCar,
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
        let e = take_entry(ring, effects, object.word(car_word::TAG)? & 0xFF_FFFF, 0x15)?;
        let put = |o: u32, v: u32| crate::effects::ring_entries::put(ring, e, o, v);
        let car = emitting_car(effects)?;
        let m = car.whole()?;
        let r = next_c5(&particles.kind21_set.side_random);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        effects.kinds.emitter.sides[0].set(a as u32);
        effects.kinds.emitter.sides[1].set(b as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            let k = 4 * k as u32;
            Ok(x87::pair_sum_80(
                a,
                matrix_cell(m, k)?,
                b,
                matrix_cell(m, 0x20 + k)?,
                wheel.place[k as usize / 4].bits(),
            ))
        })?;
        put(field::PLACE_Z, rounded(place[2]))?;
        put(field::PLACE_Y, rounded(place[1]))?;
        put(
            field::GROUND_Y,
            x87::ext_less(place[1], particles.kind21_set.lift.bits()),
        )?;
        put(field::PLACE_X, rounded(place[0]))?;
        let motion: [u32; 3] = crate::effects::try_from_fn(|k| {
            let (first, third) = (
                matrix_cell(m, 4 * k as u32)?,
                matrix_cell(m, 0x20 + 4 * k as u32)?,
            );
            let f = [0, 1, 2].map(|i| particles.kind21_set.motion_k[3 * k + i].bits());
            Ok(x87::three_products([
                [first, wheel.speed_10.bits(), f[0]],
                [third, wheel.speed_18.bits(), f[1]],
                [third, wheel.speed_c.bits(), f[2]],
            ]))
        })?;
        put(field::MOTION_Z, motion[2])?;
        put(field::MOTION_Y, motion[1])?;
        put(field::MOTION_X, motion[0])?;
        let r = next_c5(&particles.kind21_set.spin_random);
        put(field::ANGLE_RATE, (r & 0x3F).wrapping_sub(0x20))?;
        put(field::ANGLE, r.rotate_right(6) & 0xFFF)?;
        put(field::CELL, LOOK)?;
        let s = effects
            .setup
            .work
            .seed
            .get()
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        effects.setup.work.seed.set(s);
        put(field::SIZE, x87::from_int(((s & 0x3FFF) + 0x4000) as i32))?;
        put(field::LIFE, LIFE)?;
        put(
            field::ALPHA_FADE,
            x87::over_int(effects.setup.floats.two_fifty_seven.bits(), LIFE as i32),
        )?;
        put(field::ALPHA, effects.setup.floats.two_fifty_five.bits())?;
        put(field::CAR, effects.kinds.emitter.car.get())?;
        let size = car.word(car_word::GATE_BASE)?.wrapping_add(g.rate as u32);
        put(field::SIZE_GOAL, size)?;
        put(field::CAR_PACE, car.word(car_word::PACE)?)?;
        put(
            field::SIZE,
            x87::int_times(size as i32, particles.kind21_set.size_k.bits()),
        )?;
    }
    Ok(())
}

/// Kind 0x15's emitter variables (`emit_particle_21`) (IDA 0x93A312 .. 0x93A39A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Kind21Set {
    /// 0x93A312: `flt_93A312` .. `flt_93A31E`: the rate's factors
    pub rate_k: [Float; 4],
    /// 0x93A322: `dword_93A322`: the side's random
    pub side_random: Dword,
    /// 0x93A326: `flt_93A326`: the place's drop in y
    pub lift: Float,
    /// 0x93A32A: `flt_93A32A` .. `flt_93A34A`: per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [Float; 9],
    /// 0x93A34E: `dword_93A34E`: the spin's random
    pub spin_random: Dword,
    /// 0x93A352: `flt_93A352`: the size's factor
    pub size_k: Float,
    /// 0x93A356: `flt_93A35E` .. `flt_93A396`: read only by code in no function
    pub _gap_93a356: Bytes<68>,
}

impl Kind21Set {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        Kind21Set {
            rate_k: [
                Float::from_bits(0x4282AC08),
                Float::from_bits(0x37A7C5AC),
                Float::from_bits(0x3F800000),
                Float::from_bits(0x48400000),
            ],
            side_random: Dword::new(0x0),
            lift: Float::from_bits(0x47800000),
            motion_k: [
                Float::from_bits(0x42449BA6),
                Float::from_bits(0x42449BA6),
                Float::from_bits(0xC2449BA6),
                Float::from_bits(0x42449BA6),
                Float::from_bits(0x42449BA6),
                Float::from_bits(0xC2449BA6),
                Float::from_bits(0x42449BA6),
                Float::from_bits(0x42449BA6),
                Float::from_bits(0xC2449BA6),
            ],
            spin_random: Dword::new(0x0),
            size_k: Float::from_bits(0x3E4CCCCD),
            _gap_93a356: Bytes::new([
                0x0A, 0xD7, 0x23, 0x45, 0x17, 0xB7, 0x51, 0x39, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00,
                0x40, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x47, 0x6F, 0x12, 0x83, 0x42,
                0xA6, 0x9B, 0x44, 0x42, 0x4C, 0x37, 0x99, 0xC1, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B,
                0x44, 0x42, 0x4C, 0x37, 0x99, 0xC1, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B, 0x44, 0x42,
                0x4C, 0x37, 0x99, 0xC1, 0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C, 0x3E,
            ]),
        }
    }
}

impl Kind21Set {
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
        assert_eq!(size_of::<Kind21Set>(), Kind21Set::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.kind21_set)
                == crate::moved::image_bytes(Kind21Set::START, Kind21Set::LEN),
            "KIND21_SET starts differently from the image"
        );
        assert_eq!(offset_of!(Kind21Set, rate_k), 0x93A312 - Kind21Set::START);
        assert_eq!(
            offset_of!(Kind21Set, side_random),
            0x93A322 - Kind21Set::START
        );
        assert_eq!(offset_of!(Kind21Set, lift), 0x93A326 - Kind21Set::START);
        assert_eq!(offset_of!(Kind21Set, motion_k), 0x93A32A - Kind21Set::START);
        assert_eq!(
            offset_of!(Kind21Set, spin_random),
            0x93A34E - Kind21Set::START
        );
        assert_eq!(offset_of!(Kind21Set, size_k), 0x93A352 - Kind21Set::START);
    }
}
