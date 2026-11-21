//! Emits: the thin smoke trail behind the car's exhaust.
//!
//! `effects::emit_exhaust_smoke`: one ring entry of kind 0xD for a car each frame, at its model's
//! point (the looks table, three floats by the car's model).
//!
//! The entry's place is that point (x negated, y less a constant) turned by the emitting car's
//! matrix (`x87::turned_place_less_f32`), the entry's `ground_y` that y less the set's lift, then
//! moved along the motion (the matrix's third row by the strength and the set's factors) by a
//! random's low word times a constant times the frame's ticks. The strength is a float, 1 - 1 /
//! (the car's `engine_speed` by the set's factor plus 1); the life is 250 (`alpha_fade` the
//! strength by a constant over it) and `alpha` the strength by the fade constant. Refuses (before
//! anything) a null object, a null car or matrix, and a model past the table's 34 entries, when an
//! entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::effects::particles_store::Particles;
use crate::effects::paths_store::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::read_first;
use crate::race::effects_settings_store::CarEffects;

const MODELS: u32 = 34;
const LOOK: u32 = 0x3F;
const LIFE: u32 = 0xFA;

/// Why the emitter refuses, if it does, worked out without writing.
fn refusal(
    effects: &Effects,
    ring: &Ring,
    object: &CarEffects,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        // The ring full: nothing more is read.
        next_entry(ring)?;
        if !car_with_matrix(effects)? {
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
    object: &CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if let Some(why) = refusal(effects, ring, object)? {
        return Err(BadInput(why).into());
    }
    if next_entry(ring)?.is_none() {
        return Ok(());
    }
    let model = object.model.get() as usize;
    let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, 0xD)?;
    let slot = crate::effects::ring_entries::Slot::new(ring, e);
    let car = emitting_car(effects)?;
    let cells = car.whole(effects.car_blocks()?)?;
    let mut set = particles.kind13_set.borrow_mut();
    let t: [f32; 3] =
        read_first::<_, 3>(&paths.spline_head.points[3 * model..]).map(f32::from_bits);
    let [x, y, y_less, z] = x87::turned_place_less_f32(
        t,
        effects.setup.floats.particle_13_place_k,
        effects.setup.floats.seven_hundred,
        set.lift,
        car.whole(effects.car_blocks()?)?,
    );
    slot.put(|en| &mut en.place[0], x)?;
    slot.put(|en| &mut en.place[1], y)?;
    slot.put(|en| &mut en.ground_y, y_less)?;
    slot.put(|en| &mut en.place[2], z)?;
    let one = effects.setup.floats.one;
    let s = x87::one_less_inverse(&object.engine_speed, set.strength_k, one);
    ring.work.borrow_mut().whole = s.to_bits();
    let motion: [f32; 3] = crate::effects::try_from_fn(|k| {
        Ok(x87::mul2(
            matrix_float(cells, 0x20 + 4 * k as u32)?,
            s,
            set.motion_k[k],
        ))
    })?;
    slot.put(|en| &mut en.motion[0], motion[0])?;
    slot.put(|en| &mut en.motion[1], motion[1])?;
    slot.put(|en| &mut en.motion[2], motion[2])?;
    let r = (next_c5_plain(&mut set.drift_random) & 0xFFFF) as i32;
    let moved = x87::drifted(
        r,
        effects.setup.floats.over_65536,
        motion.map(f32::to_bits),
        effects.setup.clock.borrow().frame_ticks as i32,
        [x, y, z].map(f32::to_bits),
    );
    slot.put(|en| &mut en.place[0], moved[0])?;
    slot.put(|en| &mut en.place[1], moved[1])?;
    slot.put(|en| &mut en.place[2], moved[2])?;
    let r = next_c5_plain(&mut set.spin_random);
    slot.put(|en| &mut en.angle_rate, (r & 0x3F).wrapping_sub(0x20))?;
    slot.put(|en| &mut en.angle, r.rotate_right(6) & 0xFFF)?;
    slot.put(|en| &mut en.cell, LOOK)?;
    slot.put(|en| &mut en.life, LIFE)?;
    slot.put(
        |en| &mut en.alpha_fade,
        f32::from_bits(bh::st(
            bh::ld(s) * bh::ld(effects.setup.floats.two_fifty_seven) / bh::int(LIFE as i32),
        )),
    )?;
    slot.put(
        |en| &mut en.alpha,
        x87::mul(s, effects.setup.floats.two_fifty_six).to_bits(),
    )?;
    slot.put(|en| &mut en.car, effects.kinds.emitter.borrow().car)?;
    let size = x87::times_plus_to_int(set.size_k[0], s, set.size_k[1]);
    slot.put(|en| &mut en.size_goal, size as u32)?;
    slot.put(|en| &mut en.car_pace, car.pace.get())?;
    slot.put(
        |en| &mut en.size,
        x87::int_times(size, set.size_k[2]).to_bits(),
    )?;
    Ok(())
}
