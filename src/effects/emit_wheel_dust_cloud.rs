//! Emits: the soft, fading dust (snow powder on snow) cloud that trails behind a wheel.
//!
//! `effects::emit_wheel_dust_cloud`: one ring entry of kind 6 from a wheel, when its flags
//! (+0x20, kept as the wheel flags) have 0xC0 and the rate gate passes (`effects/emitters.rs`, the
//! wheel's +0xC less +0x18 by the set's rate factors).
//!
//! The entry sits at the wheel's place (+0x3C ..) plus a random side, less the set's lift in y at
//! +0x28; its motion is three sums of products of the wheel's speeds; its life comes from the lives
//! table by the flags' bits 6 and 7 (+0x84; +0x78 a constant over it) and +0x74 is the fade
//! constant. Refuses (before anything) a null wheel, object or matrix where read, and flags of
//! 0x100 or more when an entry is made (the life would be read past the lives table's four).

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::mask_store::Masks;
use crate::effects::owner::Effects;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::scene::matrix3::rounded_value;
use crate::session::{PlainDword, PlainFloat};

/// The motion: per axis (x, y, z) the three (factor, wheel speed) pairs; the scales are
/// `WheelFadeSet.motion_k[3 * axis + n]`.
/// A wheel record's speed figure.
type Speed = fn(&MarkRecord) -> &PlainFloat;

fn motion_factors(effects: &Effects) -> [[(&PlainFloat, Speed); 3]; 3] {
    [
        [
            (&effects.kinds.emitter.side_a[0], |w| &w.local_velocity_x),
            (&effects.kinds.emitter.side_b[0], |w| &w.local_velocity_z),
            (&effects.kinds.emitter.side_b[0], |w| &w.driven_speed),
        ],
        [
            (&effects.kinds.emitter.side_a[1], |w| &w.local_velocity_x),
            (&effects.kinds.emitter.side_b[1], |w| &w.local_velocity_z),
            (&effects.kinds.emitter.side_b[1], |w| &w.driven_speed),
        ],
        [
            (&effects.kinds.emitter.side_a[2], |w| &w.local_velocity_x),
            (&effects.kinds.emitter.side_b[2], |w| &w.local_velocity_z),
            (&effects.kinds.emitter.side_b[2], |w| &w.driven_speed),
        ],
    ]
}

/// Why the function refuses, if it does, worked out without writing.
fn refusal(
    effects: &Effects,
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &CarEffects,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        let flags = wheel.flags.get();
        if flags & 0xC0 == 0 {
            return Ok(None);
        }
        if !gate(
            effects,
            wheel,
            object,
            &wheel.driven_speed,
            &wheel.local_velocity_z,
            &particles.kind6_set.rate_k,
        )?
        .passes
            || next_entry(ring)?.is_none()
        {
            return Ok(None);
        }
        if !car_with_matrix(effects)? {
            return Ok(Some("a null car object or matrix"));
        }
        if flags >> 6 > 3 {
            return Ok(Some(
                "wheel flags of 0x100 or more: the life read past dword_939B76",
            ));
        }
        None
    })
}

pub fn emit_wheel_dust_cloud(
    masks: &Masks,
    particles: &Particles,
    ring: &Ring,
    wheel: &MarkRecord,
    object: &CarEffects,
    effects: &Effects,
) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(effects, particles, ring, wheel, object)? {
            return Err(BadInput(why).into());
        }
        let flags = wheel.flags.get();
        masks.surface.wheel_flags.set(flags);
        if flags & 0xC0 == 0 {
            return Ok(());
        }
        let g = gate(
            effects,
            wheel,
            object,
            &wheel.driven_speed,
            &wheel.local_velocity_z,
            &particles.kind6_set.rate_k,
        )?;
        ring.work.rate.set(g.rate as u32);
        if let Some(count) = g.count {
            wheel.pace.set(count);
        }
        if !g.passes || next_entry(ring)?.is_none() {
            return Ok(());
        }
        let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, 6)?;
        let slot = crate::effects::ring_entries::Slot::new(ring, e);
        let car = emitting_car(effects)?;
        let m = car.whole(effects.car_blocks()?)?;
        let r = next_c5(&particles.kind6_set.side_random);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        effects.kinds.emitter.sides[0].set(a as u32);
        effects.kinds.emitter.sides[1].set(b as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k = 4 * k as u32;
                x87::pair_sum_80(
                    a,
                    matrix_float(m, k)?,
                    b,
                    matrix_float(m, 0x20 + k)?,
                    &wheel.place[k as usize / 4],
                )
            })
        })?;
        slot.put(|en| &mut en.place[2], rounded_value(place[2]))?;
        slot.put(|en| &mut en.place[1], rounded_value(place[1]))?;
        slot.put(
            |en| &mut en.ground_y,
            x87::ext_less(place[1], &particles.kind6_set.lift),
        )?;
        slot.put(|en| &mut en.place[0], rounded_value(place[0]))?;
        let factors = motion_factors(effects);
        let motion: [f32; 3] = crate::effects::try_from_fn(|a| {
            Ok({
                x87::three_products(crate::effects::try_from_fn(|n| {
                    Ok({
                        let (f, s) = factors[a][n];
                        [
                            f.get(),
                            s(wheel).get(),
                            particles.kind6_set.motion_k[3 * a + n].get(),
                        ]
                    })
                })?)
            })
        })?;
        slot.put(|en| &mut en.motion[2], motion[2])?;
        slot.put(|en| &mut en.motion[1], motion[1])?;
        slot.put(|en| &mut en.motion[0], motion[0])?;
        let r = next_c5(&particles.kind6_set.spin_random);
        slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
        slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
        slot.put(|en| &mut en.cell, flags & 0x3F)?;
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
        let life: u32 = masks.surface.lives_by_flags[(flags >> 6) as usize].get();
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
        let size = car.rate.get().wrapping_add(g.rate as u32);
        slot.put(|en| &mut en.size_goal, size)?;
        slot.put(|en| &mut en.car_pace, car.pace.get())?;
        slot.put(
            |en| &mut en.size,
            x87::int_times_plus(
                size as i32,
                &particles.kind6_set.size_k[0],
                &particles.kind6_set.size_k[1],
            )
            .to_bits(),
        )?;
    }
    Ok(())
}

/// Kind 6's emitter variables (`emit_wheel_dust_cloud`).
#[derive(rc2k_derive::Cells)]
pub struct WheelFadeSet {
    /// The rate's factors
    pub rate_k: [PlainFloat; 4],
    /// The side's random
    pub side_random: PlainDword,
    /// The place's drop in y
    pub lift: PlainFloat,
    /// Per axis (x, y, z) the scales of the three products (wheel +0x10, +0x18, +0xC)
    pub motion_k: [PlainFloat; 9],
    /// The spin's random
    pub spin_random: PlainDword,
    /// The size's factors
    pub size_k: [PlainFloat; 2],
}

impl WheelFadeSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WheelFadeSet {
            rate_k: [
                PlainFloat::new(65.336),
                PlainFloat::new(0.0002),
                PlainFloat::new(1.0),
                PlainFloat::new(196608.0),
            ],
            side_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            motion_k: [
                PlainFloat::new(65.536),
                PlainFloat::new(-49.152),
                PlainFloat::new(26.152),
                PlainFloat::new(65.536),
                PlainFloat::new(-49.152),
                PlainFloat::new(26.152),
                PlainFloat::new(65.536),
                PlainFloat::new(-49.152),
                PlainFloat::new(26.152),
            ],
            spin_random: PlainDword::new(0x0),
            size_k: [PlainFloat::new(0.3), PlainFloat::new(8192.0)],
        }
    }
}
