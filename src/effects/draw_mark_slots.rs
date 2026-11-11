//! `effects::draw_mark_slots` (0x4C969A): the mark slots drawn, one square sprite each. When
//! the settings' +0x8C is set, `effects::add_mark_slot` first (with the settings' +0x64 and
//! the view place); `scene::set_texture` with the marks' texture. Then for each slot of the
//! mark slots from 0x20 down to 0 (one past the 0x20 that `add_mark_slot` fills: section 16)
//! whose life (+0) is above 0:
//! - the life less the frame's ticks;
//! - its point (+4, 12 bytes each) as z, y, -x (z times a scale, (y - a lift) times a scale,
//!   -x times a scale), turned by the slot's matrix (+8) and the view (`scene::rotate_point`
//!   twice);
//! - one of eight corners by the slots' random (the random times 0xC5 plus 0x5D; entry 0x50
//!   plus its low three bits): the four vertices' texture places (its words, times a
//!   constant), colours -1;
//! - unless the turned z is at or below the nearest depth (a signed compare of the bits): the
//!   point projected (the view scale over z; centre the view's middle), half the square q
//!   times a size factor (slot 0's +0x10 for every slot: section 16), the four vertices
//!   around it, their z and rhw (1 / z times a constant), and `scene::draw_fan` (the vertex
//!   count as it is).
//!
//! Works on the effects settings.
//!
//! Caller: `effects::run_frame` Refuses (before anything) null settings and a
//! live slot with a null matrix.

use crate::d3d::vertices_store::VERTICES;
use crate::effects::mark_slots::slot_matrix;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::read_on_in;
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

/// `add_mark_slot`, `scene::set_texture`,
/// `scene::draw_fan`.
pub fn draw_mark_slots(
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
    crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[202].get());
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
        let at = |n: usize| {
            read_on_in(
                &paths.spline_head,
                &paths.spline_head.points,
                p + n,
                "a spline point",
            )
        };
        let [a, b, c] = [at(0)? ^ 0x8000_0000, at(1)?, at(2)?];
        let k4 = effects
            .blocks
            .slot_tuning
            .place_k
            .each_ref()
            .map(super::super::session::Float::bits);
        let z = x87::mul(c, k4[0]);
        let y = x87::sub_mul(b, k4[1], k4[2]);
        let x = x87::mul(a, k4[3]);
        let once = rotate_point(
            slot_matrix(&race.frame, s.matrix.get())?,
            [extended(z), extended(y), extended(x)],
        );
        let twice = rotate_point(view.view_matrix, once);
        let [z, y, x] = twice.map(rounded);
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
        let uv_k: u32 = effects.frame.draw_work.uv_k.bits();
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            vertex.u.set_bits(x87::int_times(word(u) as i32, uv_k));
            vertex.v.set_bits(x87::int_times(word(v) as i32, uv_k));
        }
        for n in 0..4 {
            VERTICES.vertices[n].colour.set(0xFFFF_FFFFu32);
        }
        if (x as i32) <= (race.course.numbers.k4096_a.bits() as i32) {
            continue;
        }
        let [px, py, half] = x87::slot_project(
            crate::scene::polygon_work::front_end()
                .window
                .view_scale
                .bits(),
            x,
            z,
            crate::scene::polygon_work::front_end().window.view_centre[0].bits(),
            y,
            crate::scene::polygon_work::front_end().window.view_centre[1].bits(),
            effects.blocks.slots.slots[0].size.bits(),
        );
        effects.blocks.slots.work[0].set(px);
        effects.blocks.slots.work[1].set(py);
        effects.blocks.slot_tuning.half.set_bits(half);
        let corners = [
            (x87::sub(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::add(py, half)),
            (x87::sub(px, half), x87::add(py, half)),
        ];
        let depth = x87::mul(x, &effects.frame.draw_work.z_k);
        let rhw = x87::reciprocal_times(x, effects.blocks.slot_tuning.rhw_k.bits());
        for (n, (vx, vy)) in corners.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            vertex.x.set_bits(vx);
            vertex.y.set_bits(vy);
            vertex.z.set_bits(depth);
            vertex.rhw.set_bits(rhw);
        }
        crate::effects::clip_quad::draw_fan();
    }
    Ok(())
}
