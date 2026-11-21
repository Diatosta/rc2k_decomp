//! `effects::emit_rolling_particles_rated`: one ring entry of kind 0x15 for the wheel kind 1, when
//! the kind-1 choice is not 0 and the rate gate passes.
//!
//! The rate (`rate` in the ring's work) is (1 - 1 / (the wheel's `driven_speed` by the set's first
//! two factors, plus its third)) by its fourth; then the gate of `effects/emitters.rs`. The entry's
//! place is the wheel's `place` plus a random side, the entry's `ground_y` that y less the set's
//! `lift`; its motion per axis the matrix's first row by the wheel's `local_velocity_x`, its third
//! by `local_velocity_z` and by `driven_speed`, each by a factor (`motion_k`); its life is 1000;
//! its `size` is the rate sum by `size_k` (after a first value by the effects' random, which still
//! steps). Refuses (before anything) a null wheel, a null object, and a null car or matrix when an
//! entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::particles_store::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::rounded_value;
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

const LOOK: u32 = 0x34;
const LIFE: u32 = 0x3E8;

/// The rate, from the wheel's `driven_speed`.
fn rate(
    particles: &Particles,
    wheel: &MarkRecord,
    effects: &crate::effects::owner::Effects,
) -> Result<i32, Refused> {
    Ok({
        let [k1, k2, k3, k4] = particles.kind21_set.borrow().rate_k;
        x87::speed_rate_to_int(
            &wheel.driven_speed,
            k1,
            k2,
            k3,
            effects.setup.floats.one,
            k4,
        )
    })
}

/// Why the emitter refuses, if it does, worked out without writing.
fn refusal(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &CarEffects,
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

pub(super) fn emit_rolling_particles_rated(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(particles, ring, wheel, object, effects)? {
            return Err(BadInput(why).into());
        }
        let g = gate_with(wheel, object, rate(particles, wheel, effects)?)?;
        ring.work.borrow_mut().rate = g.rate as u32;
        if let Some(count) = g.count {
            wheel.pace.set(count);
        }
        if !g.passes || next_entry(ring)?.is_none() {
            return Ok(());
        }
        let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, 0x15)?;
        let slot = crate::effects::ring_entries::Slot::new(ring, e);
        let car = emitting_car(effects)?;
        let m = car.whole(effects.car_parts()?)?;
        let mut set = particles.kind21_set.borrow_mut();
        let r = next_c5_plain(&mut set.side_random);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        effects.kinds.emitter.borrow_mut().sides[0] = a as u32;
        effects.kinds.emitter.borrow_mut().sides[1] = b as u32;
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
        slot.put(|en| &mut en.ground_y, x87::ext_less(place[1], set.lift))?;
        slot.put(|en| &mut en.place[0], rounded_value(place[0]))?;
        let motion: [f32; 3] = crate::effects::try_from_fn(|k| {
            let (first, third) = (
                matrix_float(m, 4 * k as u32)?,
                matrix_float(m, 0x20 + 4 * k as u32)?,
            );
            let f = [0, 1, 2].map(|i| set.motion_k[3 * k + i]);
            Ok(x87::three_products([
                [first, wheel.local_velocity_x.get(), f[0]],
                [third, wheel.local_velocity_z.get(), f[1]],
                [third, wheel.driven_speed.get(), f[2]],
            ]))
        })?;
        slot.put(|en| &mut en.motion[2], motion[2])?;
        slot.put(|en| &mut en.motion[1], motion[1])?;
        slot.put(|en| &mut en.motion[0], motion[0])?;
        let r = next_c5_plain(&mut set.spin_random);
        slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
        slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
        slot.put(|en| &mut en.cell, LOOK)?;
        let s = effects
            .setup
            .work
            .borrow()
            .seed
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        effects.setup.work.borrow_mut().seed = s;
        slot.put(
            |en| &mut en.size,
            x87::from_int(((s & 0x3FFF) + 0x4000) as i32).to_bits(),
        )?;
        slot.put(|en| &mut en.life, LIFE)?;
        slot.put(
            |en| &mut en.alpha_fade,
            x87::over_int(effects.setup.floats.two_fifty_seven, LIFE as i32),
        )?;
        slot.put(
            |en| &mut en.alpha,
            effects.setup.floats.two_fifty_five.to_bits(),
        )?;
        slot.put(|en| &mut en.car, effects.kinds.emitter.borrow().car)?;
        let size = car.rate.get().wrapping_add(g.rate as u32);
        slot.put(|en| &mut en.size_goal, size)?;
        slot.put(|en| &mut en.car_pace, car.pace.get())?;
        slot.put(
            |en| &mut en.size,
            x87::int_times(size as i32, set.size_k).to_bits(),
        )?;
    }
    Ok(())
}
