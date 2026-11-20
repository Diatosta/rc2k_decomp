//! `effects::run_ribbons`: the effects' ring walked for its streaks (kind 0x12) when the first car's
//! `streaks_on` is set. Each streak's life runs down by the frame's ticks (dead when it borrows
//! or its y is below its ground); a new one has its place turned by the view into its last place and is
//! moved once, then moved each frame (the motion's y gains a constant a tick). Each is drawn by
//! `effects::draw_ribbon` with a colour from the streak colours (or the second ramp when the
//! effects' kind switch is set). A dead entry is moved to just before the cursor.
//!
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`), over the entries and the
//! game's variables; the fog likewise. Refuses, before anything (the walk simulated), null
//! settings, a null entry or link on the way, and a colour index out of its table.

use crate::effects::mask_store::Masks;
use crate::effects::ring_entries::{entry, ld, set, st, walk_ring};
use crate::effects::ring_store::Ring;
use crate::effects::setup_store::MarksClock;
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::scene::fog_at::fog_at;

const STREAK: u32 = 0x12;

/// A new streak's place turned into its last place, and a first move.
fn first_frame(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    e: u32,
    effects: &crate::effects::owner::Effects,
) {
    first_frame_x87(
        view_matrix,
        ring,
        e,
        &effects.setup.clock.borrow(),
        ring.kinds.ribbon_a,
    );
}

/// The place moved by the motion, the motion's y by a constant.
fn move_(ring: &Ring, e: u32, effects: &crate::effects::owner::Effects) {
    move_x87(ring, e, &effects.setup.clock.borrow(), ring.kinds.ribbon_b);
}

/// The place turned by the view into `DrawWork.turned[0]` ...
fn turn_place(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    e: u32,
    effects: &crate::effects::owner::Effects,
) {
    turn_place_x87(
        view_matrix,
        ring,
        e,
        &mut effects.frame.draw_work.borrow_mut(),
    );
}

/// The size of the colour table in use: 128 colours (the surface's) while the kind floats'
/// `other` is not 0, else 257.
fn colour_count(effects: &crate::effects::owner::Effects) -> i32 {
    if effects.setup.kind_floats.borrow().other != 0 {
        128
    } else {
        257
    }
}

/// The colour `i` of the table in use (`i` below `colour_count`).
fn colour_at(
    masks: &Masks,
    ring: &Ring,
    i: usize,
    effects: &crate::effects::owner::Effects,
) -> u32 {
    if effects.setup.kind_floats.borrow().other != 0 {
        masks.surface.borrow().colours[i]
    } else {
        ring.colours.colours[i]
    }
}

/// Why it refuses, if it does: the walk worked out first, nothing written.
fn refusal(
    ring: &Ring,
    effects: &crate::effects::owner::Effects,
    fog: &crate::scene::fog::Fog,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        let Ok(marks) = effects.settings() else {
            return Ok(Some("no marks settings"));
        };
        let ticks: u32 = effects.setup.clock.borrow().frame_ticks;
        let mut e: u32 = ring.work.borrow().cursor;
        let count = ring.work.borrow().count as i32;
        if count > 0 && (e == 0 || entry(ring, e)?.prev == 0) {
            return Ok(Some("a null ring entry"));
        }
        for _ in 0..count.max(0) {
            if e == 0 || entry(ring, e)?.prev == 0 || entry(ring, e)?.next == 0 {
                return Ok(Some("a null ring entry or link"));
            }
            if marks.cars[0].streaks_on.get() != 0 && entry(ring, e)?.kind == STREAK {
                let life = entry(ring, e)?.life;
                if life >= ticks
                    && x87::compare_status(entry(ring, e)?.place[1], entry(ring, e)?.ground_y)
                        != 0x100
                {
                    if fog_at(fog, 0).is_err() {
                        return Ok(Some("the fog table is not there"));
                    }
                    let i = x87::int_times_to_int(
                        life.wrapping_sub(ticks) as i32,
                        entry(ring, e)?.alpha_fade,
                    );
                    if !(0..colour_count(effects)).contains(&i) {
                        return Ok(Some("a streak's colour index past its table"));
                    }
                }
            }
            e = entry(ring, e)?.next;
        }
        None
    })
}

/// The streak step at e (see the header): whether the entry dies.
fn streak(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    e: u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    fog: &crate::scene::fog::Fog,
) -> Result<bool, Refused> {
    let t: u32 = effects.setup.clock.borrow().frame_ticks;
    let life = entry(ring, e)?.life;
    set(ring, e, life.wrapping_sub(t), |en| &mut en.life)?;
    if life < t {
        return Ok(true);
    }
    if x87::compare_status(entry(ring, e)?.place[1], entry(ring, e)?.ground_y) == 0x100 {
        return Ok(true);
    }
    if entry(ring, e)?.place[0].to_bits() == 0 {
        first_frame(view_matrix, ring, e, effects);
    }
    move_on(ring, e, effects);
    effects.frame.draw_work.borrow_mut().second[0] = entry(ring, e)?.last_place[0];
    effects.frame.draw_work.borrow_mut().second[1] = entry(ring, e)?.last_place[1];
    effects.frame.draw_work.borrow_mut().second[2] = entry(ring, e)?.last_place[2];
    turn_place(view_matrix, ring, e, effects);
    let marks = effects.settings()?;
    if marks.own_marks_shown.get() != 0 || marks.other_marks_shown.get() == 1 {
        set(ring, e, effects.frame.draw_work.borrow().turned[0], |en| {
            &mut en.last_place[0]
        })?;
        set(ring, e, effects.frame.draw_work.borrow().turned[1], |en| {
            &mut en.last_place[1]
        })?;
        set(ring, e, effects.frame.draw_work.borrow().turned[2], |en| {
            &mut en.last_place[2]
        })?;
    }
    buffer.vertices[0]
        .specular
        .set(fog_at(fog, entry(ring, e)?.last_place[2].to_bits())?);
    let i = x87::int_times_to_int(entry(ring, e)?.life as i32, entry(ring, e)?.alpha_fade);
    ring.work.borrow_mut().whole = i as u32;
    let colour = colour_at(&effects.masks, ring, i as usize, effects);
    crate::effects::draw_ribbon::draw_ribbon(
        buffer,
        &effects.masks,
        0x20,
        i as u32,
        0x800,
        colour,
        scene,
        effects,
    )?;
    Ok(false)
}

fn move_on(ring: &Ring, e: u32, effects: &crate::effects::owner::Effects) {
    move_(ring, e, effects);
}

pub fn run_ribbons(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    let (view_matrix, fog) = (view.view_matrix, view.fog);
    if let Some(why) = refusal(ring, effects, fog)? {
        return Err(BadInput(why).into());
    }
    walk_ring(ring, |e| {
        // The settings' streak switch: the first car record's `streaks_on`.
        Ok(effects.settings()?.cars[0].streaks_on.get() != 0
            && entry(ring, e)?.kind == STREAK
            && streak(buffer, view_matrix, ring, e, scene, effects, fog)?)
    })
}

use crate::effects::frame_state_store::DrawWork;
use crate::math::x87_float as bh;
use crate::math::x87_float::int;

/// The entry's place turned by the view matrix (its rows' x, y, z columns plus the translation
/// in cells 12 .. 14), in the x87's order: the three results unrounded to a stored float.
fn view_turned(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    e: u32,
) -> [bh::X; 3] {
    let t0 = ld(ring, e, |en| en.place[2]);
    let t1 = ld(ring, e, |en| en.place[1]);
    let t2 = ld(ring, e, |en| en.place[0]);
    let t3 = view_matrix[0].ld();
    let t4 = t3 * t2;
    let t5 = view_matrix[4].ld();
    let t6 = t5 * t1;
    let t7 = view_matrix[8].ld();
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + view_matrix[12].ld();
    let t12 = view_matrix[1].ld();
    let t13 = t12 * t2;
    let t14 = view_matrix[5].ld();
    let t15 = t14 * t1;
    let t16 = view_matrix[9].ld();
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + view_matrix[13].ld();
    let t21 = view_matrix[2].ld();
    let t22 = t2 * t21;
    let t23 = view_matrix[6].ld();
    let t24 = t1 * t23;
    let t25 = view_matrix[10].ld();
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + view_matrix[14].ld();
    [t11, t20, t29]
}

/// `first_frame`'s arithmetic as plain Rust on the x87 model, in the same order.
fn first_frame_x87(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    e: u32,
    c: &MarksClock,
    ribbon_a: f32,
) {
    let [turned_x, turned_y, turned_z] = view_turned(view_matrix, ring, e);
    st(ring, e, turned_x, |en| &mut en.last_place[0]);
    st(ring, e, turned_y, |en| &mut en.last_place[1]);
    st(ring, e, turned_z, |en| &mut en.last_place[2]);
    move_x87(ring, e, c, ribbon_a);
}

/// `move_`'s x87 block as plain Rust on the x87 model (16 instructions), same order and memory
/// places: the place gains the motion times the ticks, then the motion's y gains `y_gain` times the
/// ticks.
fn move_x87(ring: &Ring, e: u32, c: &MarksClock, y_gain: f32) {
    let ticks = int(c.frame_ticks as i32);
    for axis in 0..3 {
        st(
            ring,
            e,
            ld(ring, e, |en| en.motion[axis]) * ticks + ld(ring, e, |en| en.place[axis]),
            |en| &mut en.place[axis],
        );
    }
    st(
        ring,
        e,
        bh::ld(y_gain) * ticks + ld(ring, e, |en| en.motion[1]),
        |en| &mut en.motion[1],
    );
}

/// `turn_place`'s arithmetic as plain Rust on the x87 model, in the same order.
fn turn_place_x87(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    e: u32,
    dw: &mut DrawWork,
) {
    let [x, y, z] = view_turned(view_matrix, ring, e);
    dw.turned[0] = bh::stf(x);
    dw.turned[1] = bh::stf(y);
    dw.turned[2] = bh::stf(z);
}

#[cfg(test)]
mod tests;
