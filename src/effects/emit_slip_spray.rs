//! One ring entry of kind 5 from a wheel whose `local_velocity_z` is not 0 (spray from a slipping
//! wheel).
//!
//! The strength is |the wheel's `local_velocity_z`| less its `local_velocity_x` on even wheels and
//! plus it on odd ones. The entry starts at the wheel's place less the car matrix's, plus a random
//! side, and moves along the matrix's first and third rows by the set's factors, jittered by
//! `effects::jitter_point`. The entry's `size` is ((`jitter_point`'s third result >> 2) & 0xFFF) +
//! 0x7800 (a slip kept as is); the life is 5000. The body is shared with kind 0xB (`JitterSet`,
//! `emit_slip_spray_with`). Refuses (before anything) a null wheel, object, car or matrix where
//! read.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

/// One emitter's variables (kind 5's and kind 0xB's code is the same but for these).
pub struct JitterSet<'a> {
    pub kind: u32,
    /// Whether the rounded |speed| must be over 1 (kind 0xB) or the speed not 0 (kind 5).
    pub over_one: bool,
    pub place_random: &'a PlainDword,
    /// The motion's factors (x, y, z); `lift` the y's.
    pub motion_k: [&'a PlainFloat; 3],
    pub lift: &'a PlainFloat,
    pub spread: &'a PlainFloat,
    /// The third row's factors (z, y, x).
    pub roll_k: [&'a PlainFloat; 3],
    pub spin: &'a PlainFloat,
    pub life: u32,
}

/// Kind 5's variables.
pub fn fifth(effects: &Effects) -> JitterSet<'_> {
    JitterSet {
        kind: 5,
        over_one: false,
        place_random: &effects.particles.kind5_set.place_random,
        motion_k: [
            &effects.particles.kind5_set.motion_x,
            &effects.particles.kind5_set.motion_y,
            &effects.particles.kind5_set.motion_z,
        ],
        lift: &effects.particles.kind5_set.lift,
        spread: &effects.particles.kind5_set.spread,
        roll_k: [
            &effects.particles.kind5_set.roll_k[0],
            &effects.particles.kind5_set.roll_k[1],
            &effects.particles.kind5_set.roll_k[2],
        ],
        spin: &effects.particles.kind5_set.spin,
        life: 0x1388,
    }
}

/// Why the emitter refuses, if it does, worked out without writing.
pub fn refusal(
    effects: &Effects,
    set: &JitterSet<'_>,
    wheel: &MarkRecord,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !emits(set, wheel)? {
            return Ok(None);
        }
        // The ring full: nothing more is read.
        next_entry(&effects.ring)?;
        if !car_with_matrix(effects)? {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

/// Whether the wheel's `local_velocity_z` passes the set's test.
fn emits(set: &JitterSet<'_>, wheel: &MarkRecord) -> Result<bool, Refused> {
    Ok({
        let speed = wheel.local_velocity_z.get().abs();
        if set.over_one {
            x87::to_int(speed) > 1
        } else {
            x87::loaded(speed).to_bits() != 0
        }
    })
}

pub(super) fn emit_slip_spray(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &CarEffects,
) -> Result<(), Refused> {
    emit_slip_spray_with(effects, &fifth(effects), wheel, object)
}

/// The shared body, on `set`'s variables.
pub(super) fn emit_slip_spray_with(
    effects: &Effects,
    set: &JitterSet<'_>,
    wheel: &MarkRecord,
    object: &CarEffects,
) -> Result<(), Refused> {
    let (ring, kinds) = (&effects.ring, &effects.kinds);
    {
        if let Some(why) = refusal(effects, set, wheel)? {
            return Err(BadInput(why).into());
        }
        let strength = x87::loaded(&wheel.local_velocity_z).abs();
        kinds.emitter.strength.set(strength);
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
        let m = car.whole(effects.car_blocks()?)?;
        let r = next_c5(set.place_random);
        let (a, b) = (((r & 0x3FFF) as i32) - 0x2000, (r & 0x3FFF) as i32);
        kinds.emitter.sides[0].set(a as u32);
        kinds.emitter.sides[1].set(b as u32);
        let place: [f32; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k = 4 * k as u32;
                x87::pair_sum_less(
                    a,
                    matrix_float(m, k)?,
                    b,
                    matrix_float(m, 0x20 + k)?,
                    &wheel.place[k as usize / 4],
                    matrix_float(m, 0x30 + k)?,
                )
            })
        })?;
        slot.put(|en| &mut en.place[2], place[2])?;
        slot.put(|en| &mut en.ground_y, place[1])?;
        slot.put(|en| &mut en.place[1], place[1])?;
        slot.put(|en| &mut en.place[0], place[0])?;
        let motion: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let lift = (k == 1).then_some(&set.lift);
                x87::times_int_plus_times_80(
                    matrix_float(m, 4 * k as u32)?,
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
        let speed = wheel.local_velocity_z.get();
        // (Motion z first, then y, then x.)
        for (n, moved) in moved.into_iter().enumerate() {
            let row = matrix_float(m, 0x28 - 4 * n as u32)?;
            slot.put(
                |en| &mut en.motion[2 - n],
                x87::product_plus(row, set.roll_k[n], speed, rounded_value(moved)),
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
