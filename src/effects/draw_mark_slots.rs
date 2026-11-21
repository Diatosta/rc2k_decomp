//! `effects::draw_mark_slots`: the mark slots drawn, one square sprite each. When the first car's
//! `gear_notice` is set, `effects::add_mark_slot` first takes a slot (with that car's `model` and
//! the view place); `scene::set_texture` sets the marks' texture. Then each of the 0x20 slots, the
//! last first, whose `life` is above 0 loses the frame's ticks and is drawn: its point (three
//! floats, taken as z, y, -x, each scaled) is turned by the slot's `matrix` and the view
//! (`scene::rotate_point` twice); one of eight corner pictures is chosen by the slots' random
//! (entry 0x50 plus its low three bits) for the texture places; unless the turned z is at or below
//! the nearest depth (a signed compare of the bits), the point is projected and the square drawn
//! around it with `scene::draw_fan`. Its half side comes from slot 0's `size` for every slot (a
//! slip kept as is).
//!
//! Refuses (before anything) null settings and a live slot with a null matrix.

use crate::effects::mark_slots::slot_matrix;
use crate::effects::paths_store::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::rotate_point::rotate_point_by;
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// Why it refuses, if it does, worked out before anything is written.
fn refusal(effects: &crate::effects::owner::Effects) -> Option<&'static str> {
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Some("no marks settings");
        }
        for slot in &effects.blocks.slots.borrow().slots {
            if (slot.life as i32) > 0 && slot.matrix == 0 {
                return Some("a live mark slot with a null matrix");
            }
        }
        None
    }
}

/// `add_mark_slot`, `scene::set_texture`, `scene::draw_fan`.
pub fn draw_mark_slots(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if let Some(why) = refusal(effects) {
        return Err(BadInput(why).into());
    }
    // The first car record's `gear_notice` and `model`.
    let first = &effects.settings()?.cars[0];
    if first.gear_notice.get() != 0 {
        crate::effects::add_mark_slot::add_mark_slot(
            first.model.get(),
            crate::effects::mark_slots::VIEW_PLACE_MATRIX,
            effects,
        );
    }
    crate::scene::set_texture::set_texture(
        scene.scratch.game_scratch(),
        scene,
        scene.textures.kinds.borrow().kind_first_slots[202],
    );
    let count = effects.blocks.slots.borrow().slots.len();
    for index in (0..count).rev() {
        let (life, point, matrix) = {
            let slots = effects.blocks.slots.borrow();
            let slot = &slots.slots[index];
            (slot.life, slot.point, slot.matrix)
        };
        if (life as i32) <= 0 {
            continue;
        }
        effects.blocks.slots.borrow_mut().slots[index].life =
            life.wrapping_sub(effects.setup.clock.borrow().frame_ticks);
        let p = point as usize * 3;
        // A slot's point is the first car's model, which `share_car_state` took from the 34 light
        // records: the three floats are inside the 34 models' points.
        let at = |n: usize| {
            paths
                .spline_head
                .borrow()
                .points
                .get(p + n)
                .copied()
                .ok_or(BadInput("a mark slot's point past the car models'"))
        };
        let [a, b, c] = [-at(0)?, at(1)?, at(2)?];
        let k4 = effects.blocks.slot_tuning.borrow().place_k;
        let z = x87::mul(c, k4[0]);
        let y = x87::sub_mul(b, k4[1], k4[2]);
        let x = x87::mul(a, k4[3]);
        let once = rotate_point_by(
            &slot_matrix(&race.frame, matrix)?,
            [extended(z), extended(y), extended(x)],
        );
        let twice = rotate_point_by(&view.view_matrix, once);
        let [z, y, x] = twice.map(rounded_value);
        effects.blocks.slots.borrow_mut().work = [z, y, x];
        let r = {
            let mut tuning = effects.blocks.slot_tuning.borrow_mut();
            tuning.draw_random = tuning.draw_random.wrapping_mul(0xC5).wrapping_add(0x5D);
            tuning.draw_random
        };
        let corner = &effects.blocks.corners.corners[((r & 7) + 0x50) as usize];
        let word = |off: u32| u32::from(corner.words[off as usize / 2].get());
        let uv_k = effects.frame.draw_work.borrow().uv_k;
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = &buffer.vertices[n];
            vertex.u.set(x87::int_times(word(u) as i32, uv_k));
            vertex.v.set(x87::int_times(word(v) as i32, uv_k));
        }
        for item in buffer.vertices.iter().take(4) {
            item.colour.set(0xFFFF_FFFFu32);
        }
        if x87::bits_at_or_below(x, race.course.numbers.k4096_a.get()) {
            continue;
        }
        let [px, py, half] = x87::slot_project(
            scene.scratch.front_end_polygon().window.borrow().view_scale,
            x,
            z,
            scene
                .scratch
                .front_end_polygon()
                .window
                .borrow()
                .view_centre[0],
            y,
            scene
                .scratch
                .front_end_polygon()
                .window
                .borrow()
                .view_centre[1],
            effects.blocks.slots.borrow().slots[0].size,
        )
        .map(f32::from_bits);
        {
            let mut slots = effects.blocks.slots.borrow_mut();
            slots.work[0] = px;
            slots.work[1] = py;
        }
        effects.blocks.slot_tuning.borrow_mut().half = half;
        let corners = [
            (x87::sub(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::add(py, half)),
            (x87::sub(px, half), x87::add(py, half)),
        ];
        let depth = x87::mul(x, effects.frame.draw_work.borrow().z_k);
        let rhw = x87::reciprocal_times(x, effects.blocks.slot_tuning.borrow().rhw_k);
        for (n, (vx, vy)) in corners.into_iter().enumerate() {
            let vertex = &buffer.vertices[n];
            vertex.x.set(vx);
            vertex.y.set(vy);
            vertex.z.set(depth);
            vertex.rhw.set(rhw);
        }
        crate::effects::clip_quad::draw_fan(buffer, scene);
    }
    Ok(())
}
