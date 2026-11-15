//! `effects::draw_mark_slots`: the mark slots drawn, one square sprite each. When the settings'
//! +0x8C is set, `effects::add_mark_slot` first takes a slot (with the settings' +0x64 and the view
//! place); `scene::set_texture` sets the marks' texture. Then each of the 0x20 slots, the last
//! first, whose life (+0) is above 0 loses the frame's ticks and is drawn: its point (+4, 12 bytes
//! each, as z, y, -x, each scaled) is turned by the slot's matrix (+8) and the view
//! (`scene::rotate_point` twice); one of eight corner pictures is chosen by the slots' random
//! (entry 0x50 plus its low three bits) for the texture places; unless the turned z is at or below
//! the nearest depth (a signed compare of the bits), the point is projected and the square drawn
//! around it with `scene::draw_fan`. Its half side comes from slot 0's +0x10 for every slot (a slip
//! kept as is).
//!
//! Refuses (before anything) null settings and a live slot with a null matrix.

use crate::effects::mark_slots::slot_matrix;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;
use crate::session::PlainFloat;
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// Why it refuses, if it does, worked out before anything is written.
fn refusal(effects: &crate::effects::owner::Effects) -> Option<&'static str> {
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Some("no marks settings");
        }
        for slot in &effects.blocks.slots.slots {
            if (slot.life.get() as i32) > 0 && slot.matrix.get() == 0 {
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
    // The settings' +0x8C and +0x64: the first car record's +0x2C and +4.
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
        scene.textures.kinds.get().kind_first_slots[202].get(),
    );
    for s in effects.blocks.slots.slots.iter().rev() {
        if (s.life.get() as i32) <= 0 {
            continue;
        }
        s.life.set(
            s.life
                .get()
                .wrapping_sub(effects.setup.clock.frame_ticks.get()),
        );
        let p = s.point.get() as usize * 3;
        // A slot's point is the first car's model, which `share_car_state` took from the 34 light
        // records: the three floats are inside the 34 models' points.
        let at = |n: usize| {
            paths
                .spline_head
                .points
                .get(p + n)
                .map(PlainFloat::get)
                .ok_or(BadInput("a mark slot's point past the car models'"))
        };
        let [a, b, c] = [-at(0)?, at(1)?, at(2)?];
        let k4 = effects
            .blocks
            .slot_tuning
            .place_k
            .each_ref()
            .map(crate::session::PlainFloat::get);
        let z = x87::mul(c, k4[0]);
        let y = x87::sub_mul(b, k4[1], k4[2]);
        let x = x87::mul(a, k4[3]);
        let once = rotate_point(
            slot_matrix(&race.frame, s.matrix.get())?,
            [extended(z), extended(y), extended(x)],
        );
        let twice = rotate_point(view.view_matrix, once);
        let [z, y, x] = twice.map(rounded_value);
        for (n, v) in [z, y, x].into_iter().enumerate() {
            effects.blocks.slots.work[n].set(v);
        }
        let r = effects
            .blocks
            .slot_tuning
            .draw_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        effects.blocks.slot_tuning.draw_random.set(r);
        let corner = &effects.blocks.corners.corners[((r & 7) + 0x50) as usize];
        let word = |off: u32| u32::from(corner.words[off as usize / 2].get());
        let uv_k = effects.frame.draw_work.uv_k.get();
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = &buffer.vertices[n];
            vertex.u.set(x87::int_times(word(u) as i32, uv_k));
            vertex.v.set(x87::int_times(word(v) as i32, uv_k));
        }
        for item in buffer.vertices.iter().take(4) {
            item.colour.set(0xFFFF_FFFFu32);
        }
        if x87::bits_at_or_below(x, race.course.numbers.get().k4096_a.get()) {
            continue;
        }
        let [px, py, half] = x87::slot_project(
            scene.scratch.front_end_polygon().window.view_scale.get(),
            x,
            z,
            &scene.scratch.front_end_polygon().window.view_centre[0],
            y,
            &scene.scratch.front_end_polygon().window.view_centre[1],
            &effects.blocks.slots.slots[0].size,
        )
        .map(f32::from_bits);
        effects.blocks.slots.work[0].set(px);
        effects.blocks.slots.work[1].set(py);
        effects.blocks.slot_tuning.half.set(half);
        let corners = [
            (x87::sub(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::add(py, half)),
            (x87::sub(px, half), x87::add(py, half)),
        ];
        let depth = x87::mul(x, &effects.frame.draw_work.z_k);
        let rhw = x87::reciprocal_times(x, &effects.blocks.slot_tuning.rhw_k);
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
