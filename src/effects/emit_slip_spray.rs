//! `effects::emit_slip_spray` (0x4CC15D, a chunk of `effects::emit_wheel_particles` entered
//! from it and from another chunk): one ring entry of kind 5 from a wheel whose +0x18 is not
//! 0.
//!
//! Takes: the wheel record, the car object
//!
//! The strength: |the wheel's +0x18|, then less (wheel number even, sign -1) or plus (odd,
//! sign 1) its +0x10. The entry: its place the wheel's (+0x3C ..) less the car matrix's
//! (+0x30 ..) plus a random side (the matrix's first and third rows by a random's low 14
//! bits, the first less 0x2000); its motion the matrix's first row by the set's factors and
//! the sign, by the strength (the y plus a constant), moved by `effects::jitter_point` (the
//! set's spread), plus the third row by the set's factors and the wheel's +0x18; +0x54 the
//! set's spin factor times the sign; +0x58 the wheel's flags' low 6 bits; +0x60 the integer
//! ((`jitter_point`'s third result >> 2) & 0xFFF) + 0x7800 (section 16); +0x74 0xFF000000; a
//! life of 5000; +0x68 the car. Refuses (before anything) a null wheel, object, car or matrix
//! where read. The body is shared with kind 0xB's (`JitterSet`, `emit_slip_spray_with`,
//! `test_with`).

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::scene::matrix3::{extended, rounded};
use crate::session::{PlainDword, PlainFloat};

/// One emitter's variables (kind 5's and kind 0xB's code is the same but for these).
pub struct JitterSet {
    pub kind: u32,
    /// Whether the rounded |speed| must be over 1 (kind 0xB) or the speed not 0 (kind 5).
    pub over_one: bool,
    pub place_random: &'static PlainDword,
    /// The motion's factors (x, y, z); `lift` the y's.
    pub motion_k: [&'static PlainFloat; 3],
    pub lift: &'static PlainFloat,
    pub spread: &'static PlainFloat,
    /// The third row's factors (z, y, x).
    pub roll_k: [&'static PlainFloat; 3],
    pub spin: &'static PlainFloat,
    pub life: u32,
}

/// Kind 5's variables.
pub const FIFTH: JitterSet = JitterSet {
    kind: 5,
    over_one: false,
    place_random: &crate::effects::owner::EFFECTS
        .unchecked()
        .particles
        .kind5_set
        .place_random,
    motion_k: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .particles
            .kind5_set
            .motion_x,
        &crate::effects::owner::EFFECTS
            .unchecked()
            .particles
            .kind5_set
            .motion_y,
        &crate::effects::owner::EFFECTS
            .unchecked()
            .particles
            .kind5_set
            .motion_z,
    ],
    lift: &crate::effects::owner::EFFECTS
        .unchecked()
        .particles
        .kind5_set
        .lift,
    spread: &crate::effects::owner::EFFECTS
        .unchecked()
        .particles
        .kind5_set
        .spread,
    roll_k: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .particles
            .kind5_set
            .roll_k[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .particles
            .kind5_set
            .roll_k[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .particles
            .kind5_set
            .roll_k[2],
    ],
    spin: &crate::effects::owner::EFFECTS
        .unchecked()
        .particles
        .kind5_set
        .spin,
    life: 0x1388,
};

/// Why the emitter refuses, if it does, worked out without writing.
pub fn refusal(
    ring: &Ring,
    set: &JitterSet,
    wheel: &MarkRecord,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !emits(set, wheel)? {
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

/// Whether the wheel's +0x18 passes the set's test.
fn emits(set: &JitterSet, wheel: &MarkRecord) -> Result<bool, Refused> {
    Ok({
        let speed = wheel.local_velocity_z.bits() & 0x7FFF_FFFF;
        if set.over_one {
            x87::to_int(speed) > 1
        } else {
            x87::loaded(speed).to_bits() != 0
        }
    })
}

pub fn emit_slip_spray(
    ring: &Ring,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_slip_spray_with(ring, &FIFTH, wheel, object)
}

/// The shared body, on `set`'s variables.
pub fn emit_slip_spray_with(
    ring: &Ring,
    set: &JitterSet,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    let effects = crate::effects::owner::effects();
    let kinds = &effects.kinds;
    {
        if let Some(why) = refusal(ring, set, wheel)? {
            return Err(BadInput(why).into());
        }
        let strength = x87::loaded(&wheel.local_velocity_z).to_bits() & 0x7FFF_FFFF;
        kinds.emitter.strength.set_bits(strength);
        if !emits(set, wheel)? || next_entry(ring)?.is_none() {
            return Ok(());
        }
        let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, set.kind)?;
        let slot = crate::effects::ring_entries::Slot::new(ring, e);
        let (sign, strength) = if kinds.emitter.record.get() & 1 == 0 {
            (-1i32, x87::sub(strength, &wheel.local_velocity_x))
        } else {
            (1, x87::add(strength, &wheel.local_velocity_x))
        };
        kinds.emitter.sign.set(sign as u32);
        kinds.emitter.strength.set(strength);
        let car = emitting_car(effects)?;
        let m = car.whole()?;
        let r = next_c5(set.place_random);
        let (a, b) = (((r & 0x3FFF) as i32) - 0x2000, (r & 0x3FFF) as i32);
        kinds.emitter.sides[0].set(a as u32);
        kinds.emitter.sides[1].set(b as u32);
        let place: [u32; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k = 4 * k as u32;
                x87::pair_sum_less(
                    a,
                    matrix_cell(m, k)?,
                    b,
                    matrix_cell(m, 0x20 + k)?,
                    &wheel.place[k as usize / 4],
                    matrix_cell(m, 0x30 + k)?,
                )
                .to_bits()
            })
        })?;
        slot.put(|en| &mut en.place[2], place[2])?;
        slot.put(|en| &mut en.ground_y, place[1])?;
        slot.put(|en| &mut en.place[1], place[1])?;
        slot.put(|en| &mut en.place[0], place[0])?;
        let motion: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let lift = (k == 1).then(|| set.lift.bits());
                x87::times_int_plus_times_80(
                    matrix_cell(m, 4 * k as u32)?,
                    set.motion_k[k],
                    sign,
                    lift,
                    strength,
                )
            })
        })?;
        let (moved, last) = jitter_point(
            [extended(set.spread.get()), motion[2], motion[1], motion[0]],
            effects,
        );
        let speed = wheel.local_velocity_z.bits();
        // (Motion z first, then y, then x.)
        for (n, moved) in moved.into_iter().enumerate() {
            let row = matrix_cell(m, 0x28 - 4 * n as u32)?;
            slot.put(
                |en| &mut en.motion[2 - n],
                x87::product_plus(row, set.roll_k[n], speed, rounded(moved)).to_bits(),
            )?;
        }
        slot.put(
            |en| &mut en.angle_rate,
            x87::float_times_int_to_int(set.spin, sign) as u32,
        )?;
        slot.put(|en| &mut en.cell, wheel.flags.get() & 0x3F)?;
        slot.put(|en| &mut en.size, ((last >> 2) & 0xFFF) + 0x7800)?;
        slot.put(|en| &mut en.alpha, 0xFF00_0000)?;
        slot.put(|en| &mut en.life, set.life)?;
        slot.put(|en| &mut en.car, kinds.emitter.car.get())?;
    }
    Ok(())
}

/// Kind 5's emitter variables (`emit_slip_spray::FIFTH`) (IDA 0x93A23A .. 0x93A262).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SlipSpraySet {
    /// 0x93A23A: `dword_93A23A`: the place's random
    pub place_random: PlainDword,
    /// 0x93A23E: `flt_93A23E`: the motion's x factor
    pub motion_x: PlainFloat,
    /// 0x93A242: `flt_93A242`: the motion's y factor
    pub motion_y: PlainFloat,
    /// 0x93A246: `flt_93A246`: added to the motion's y
    pub lift: PlainFloat,
    /// 0x93A24A: `flt_93A24A`: the motion's z factor
    pub motion_z: PlainFloat,
    /// 0x93A24E: `flt_93A24E`: the spread `jitter_point` takes
    pub spread: PlainFloat,
    /// 0x93A252: `flt_93A252` .. `flt_93A25A`: the third row's factors (z, y, x)
    pub roll_k: [PlainFloat; 3],
    /// 0x93A25E: `flt_93A25E`: the spin's factor
    pub spin: PlainFloat,
}

impl SlipSpraySet {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SlipSpraySet {
            place_random: PlainDword::new(0x0),
            motion_x: PlainFloat::new(10.0),
            motion_y: PlainFloat::new(10.0),
            lift: PlainFloat::new(15.0),
            motion_z: PlainFloat::new(10.0),
            spread: PlainFloat::new(110.0),
            roll_k: [
                PlainFloat::new(-12.0),
                PlainFloat::new(-12.0),
                PlainFloat::new(-12.0),
            ],
            spin: PlainFloat::new(-64.0),
        }
    }
}

impl SlipSpraySet {
    pub const START: usize = 0x93A23A;
    pub const LEN: usize = 0x28;
}

#[cfg(test)]
mod kind5_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind5_set_starts_as_the_games() {
        assert_eq!(size_of::<SlipSpraySet>(), SlipSpraySet::LEN);
        assert!(
            bytes(&crate::effects::owner::effects().particles.kind5_set)
                == crate::moved::image_bytes(SlipSpraySet::START, SlipSpraySet::LEN),
            "KIND5_SET starts differently from the image"
        );
        assert_eq!(
            offset_of!(SlipSpraySet, place_random),
            0x93A23A - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, motion_x),
            0x93A23E - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, motion_y),
            0x93A242 - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, lift),
            0x93A246 - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, motion_z),
            0x93A24A - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, spread),
            0x93A24E - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, roll_k),
            0x93A252 - SlipSpraySet::START
        );
        assert_eq!(
            offset_of!(SlipSpraySet, spin),
            0x93A25E - SlipSpraySet::START
        );
    }
}
