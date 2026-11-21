//! Emits: the dust that rolls up behind a wheel on loose ground.
//!
//! `effects::emit_rolling_dust`: for the wheel kind 1, `effects::emit_rolling_particles_rated`
//! when the kind-1 choice is not 0, else `emit_particle_1`: one ring entry of kind 1.
//!
//! `emit_particle_1` grows the wheel's `pace` (a float here) by |`local_velocity_z`| +
//! |`local_velocity_x`| + |`local_velocity_z` - `driven_speed`|; when that rounded is 0x18 or more
//! it is kept as its remainder by 0x18 (as a float) and an entry may be made if the strength
//! (|`local_velocity_x`| by one factor plus |`driven_speed` - `local_velocity_z`|, by another; held
//! to at most 1) is not under the least strength (compared unrounded). The entry sits at the
//! wheel's place plus a random side, its `ground_y` that y less a lift; its motion is the car
//! matrix's first row by `local_velocity_x`, its third by `local_velocity_z` and the side factors
//! by `driven_speed`, each by a factor, moved by `effects::jitter_point`; its life and fade come
//! from the strength.
//!
//! Refuses (before anything) a null wheel, a null object where read, a null car or matrix when an
//! entry is made, and `emit_rolling_particles_rated`'s.

use crate::effects::emit_rolling_particles_rated::emit_rolling_particles_rated;
use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::owner::Effects;
use crate::effects::particles_store::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

/// The strength's two factors and its least.
fn strength_k(effects: &Effects) -> [f32; 3] {
    [
        effects.setup.floats.three_tenths,
        effects.setup.floats.thirty_second,
        effects.setup.floats.fifth,
    ]
}
/// The life's factor and base.
fn life_k(effects: &Effects) -> [f32; 2] {
    [
        effects.setup.floats.two_thousand,
        effects.setup.floats.hundred,
    ]
}
const SIZE: u32 = 0x1_0000;
const STEP: i32 = 0x18;

/// The first part of `emit_particle_1`, worked out without writing: the wheel's `pace` grown
/// (rounded and as a float), and, when an entry may be made, the strength (the float
/// kept, before the hold to 1).
struct Pace {
    grown: (i32, f32),
    strength: Option<(u32, bool)>,
}

fn pace(effects: &Effects, wheel: &MarkRecord) -> Result<Pace, Refused> {
    Ok({
        let grown = x87::abs_sum_to_int_kept(
            &wheel.local_velocity_z,
            &wheel.local_velocity_x,
            &wheel.driven_speed,
            wheel.pace.get(),
        );
        if grown.0 < STEP {
            return Ok(Pace {
                grown,
                strength: None,
            });
        }
        let [k1, k2, least] = strength_k(effects);
        let (kept, status) = x87::abs_blend_compare(
            &wheel.local_velocity_x,
            k1,
            &wheel.driven_speed,
            &wheel.local_velocity_z,
            k2,
            least,
        );
        let under = status & 0x700 == 0x100;
        Pace {
            grown,
            strength: Some((kept, under)),
        }
    })
}

/// Why `emit_particle_1` refuses, if it does, worked out without writing.
fn refusal(
    effects: &Effects,
    ring: &Ring,
    wheel: &MarkRecord,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        match pace(effects, wheel)?.strength {
            Some((_, false)) => {}
            _ => return Ok(None),
        }
        next_entry(ring)?;
        if !car_with_matrix(effects)? {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

/// The kind 1 emitter proper (see the header).
fn emit_particle_1(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &CarEffects,
    effects: &Effects,
) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(effects, ring, wheel)? {
            return Err(BadInput(why).into());
        }
        let mut set = particles.kind1_set.borrow_mut();
        let flags = wheel.flags.get();
        let p = pace(effects, wheel)?;
        ring.work.borrow_mut().whole = p.grown.0 as u32;
        wheel.pace.set(p.grown.1.to_bits());
        let Some((kept, under)) = p.strength else {
            return Ok(());
        };
        let rest = p.grown.0 % STEP;
        ring.work.borrow_mut().whole = rest as u32;
        wheel.pace.set(x87::from_int(rest).to_bits());
        ring.work.borrow_mut().whole = kept;
        if under {
            return Ok(());
        }
        let one = effects.setup.floats.one;
        if x87::compare_status(kept, one) != 0x100 {
            ring.work.borrow_mut().whole = one.to_bits();
        }
        let strength: u32 = ring.work.borrow().whole;
        let life = x87::times_plus_to_int(strength, life_k(effects)[0], life_k(effects)[1]);
        ring.work.borrow_mut().life = life as u32;
        let fade = x87::mul(strength, effects.setup.floats.two_fifty_five);
        ring.work.borrow_mut().fade = fade;
        if next_entry(ring)?.is_none() {
            return Ok(());
        }
        let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, 1)?;
        let slot = crate::effects::ring_entries::Slot::new(ring, e);
        let car = emitting_car(effects)?;
        let m = car.whole(effects.car_blocks()?)?;
        let side = (next_c5_plain(&mut set.side_random) & 0x3FFF) as i32 - 0x2000;
        effects.setup.frame.emit_side.set(side as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok(x87::plus_int_times_80(
                &wheel.place[k],
                effects.kinds.emitter.borrow().side_b[k],
                side,
            ))
        })?;
        slot.put(|en| &mut en.place[0], rounded_value(place[0]))?;
        slot.put(|en| &mut en.place[1], rounded_value(place[1]))?;
        slot.put(
            |en| &mut en.ground_y,
            x87::ext_less(place[1], effects.setup.floats.fixed_16_scale),
        )?;
        slot.put(|en| &mut en.place[2], rounded_value(place[2]))?;
        let motion: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k4 = 4 * k as u32;
                let f = [0, 1, 2].map(|i| set.motion_k[3 * k + i]);
                x87::three_products_80([
                    [matrix_float(m, k4)?, wheel.local_velocity_x.get(), f[0]],
                    [
                        matrix_float(m, 0x20 + k4)?,
                        wheel.local_velocity_z.get(),
                        f[1],
                    ],
                    [
                        effects.kinds.emitter.borrow().side_b[k],
                        wheel.driven_speed.get(),
                        f[2],
                    ],
                ])
            })
        })?;
        let (moved, _) = jitter_point(
            [extended(set.spread), motion[2], motion[1], motion[0]],
            effects,
        );
        slot.put(|en| &mut en.motion[2], rounded_value(moved[0]))?;
        slot.put(
            |en| &mut en.motion[1],
            x87::add(rounded_value(moved[1]), set.lift),
        )?;
        slot.put(|en| &mut en.motion[0], rounded_value(moved[2]))?;
        let r = next_c5_plain(&mut set.spin_random);
        slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
        slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
        slot.put(|en| &mut en.cell, flags & 0x3F)?;
        slot.put(|en| &mut en.life, life as u32)?;
        slot.put(|en| &mut en.alpha_fade, x87::over_int(fade, life))?;
        slot.put(|en| &mut en.alpha, fade.to_bits())?;
        slot.put(|en| &mut en.car, effects.kinds.emitter.borrow().car)?;
        slot.put(|en| &mut en.size_goal, SIZE)?;
        slot.put(|en| &mut en.car_pace, car.pace.get())?;
        slot.put(
            |en| &mut en.size,
            x87::int_times(SIZE as i32, set.size_k).to_bits(),
        )?;
        slot.put(|en| &mut en.size_factor, 0.0)?;
    }
    Ok(())
}

pub fn emit_rolling_dust(
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &CarEffects,
    effects: &Effects,
) -> Result<(), Refused> {
    if particles.kind1_choice.get() != 0 {
        emit_rolling_particles_rated(particles, ring, wheel, object, effects)
    } else {
        emit_particle_1(particles, ring, wheel, object, effects)
    }
}
