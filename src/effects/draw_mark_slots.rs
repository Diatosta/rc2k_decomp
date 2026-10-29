//! `effects::draw_mark_slots` (0x4C969A): the mark slots drawn, one square sprite each. When
//! the settings' +0x8C is set, `effects::add_mark_slot` first (eax the settings' +0x64, esi
//! `flt_5EA3CC`); `scene::set_texture` with `dword_5D1E10`. Then for each slot of
//! `g_markSlots` from 0x20 down to 0 (one past the 0x20 that `add_mark_slot` fills:
//! section 16) whose life (+0) is above 0:
//! - the life less `g_marksFrameTicks`;
//! - its point (+4, 12 bytes each at `flt_769312`) as z, y, -x into `dword_9352F4` ..
//!   (z times `flt_9353EE`, (y - `flt_9353F2`) times `flt_9353F6`, -x times `flt_9353FA`),
//!   turned by the slot's matrix (+8) and the view (`scene::rotate_point` twice);
//! - one of eight corners of `g_marksCorners` by the slots' random (`dword_9353FE` times
//!   0xC5 plus 0x5D; entry 0x50 plus its low three bits): the four vertices' texture places
//!   (its words, times `flt_8EB969`), colours -1;
//! - unless the turned z is at or below `flt_5C5E18` (a signed compare of the bits): the
//!   point projected (`flt_624088` over z; centre `flt_62409C`, `flt_6240A0`), half the
//!   square q times `flt_934F8C` (slot 0's +0x10 for every slot: section 16) in
//!   `flt_9353A6`, the four vertices around it, their z (times `flt_8EB96D`) and rhw (1 /
//!   z times `flt_935402`), and `scene::draw_fan` (the vertex count as it is).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `effects::run_frame` (0x4D1659). Refuses (before anything) null settings and a
//! live slot with a null matrix.

use crate::d3d::vertices_store::VERTICES;
use crate::effects::blocks::MARK_CORNERS;
use crate::effects::frame_state::DRAW_WORK;
use crate::effects::mark_slots::{SLOTS, TUNING};
use crate::effects::setup::CLOCK;
use crate::effects::splines::SPLINE_HEAD;
use crate::errors::{BadInput, Refused};
use crate::math::cells::At;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

/// Why it refuses, if it does, worked out before anything is written.
fn refusal() -> Option<&'static str> {
    {
        if crate::effects::setup::SETTINGS.pointer.get() == 0 {
            return Some("no marks settings");
        }
        for slot in &SLOTS.slots {
            if (slot.life.get() as i32) > 0 && slot.matrix.get() == 0 {
                return Some("a live mark slot with a null matrix");
            }
        }
        None
    }
}

/// The Rust version (see the header): `add_mark_slot`, `scene::set_texture`,
/// `scene::draw_fan`.
pub fn draw_mark_slots() -> Result<(), Refused> {
    if let Some(why) = refusal() {
        return Err(BadInput(why).into());
    }
    let settings: u32 = crate::effects::setup::SETTINGS.pointer.get();
    if get(settings + 0x8C)? != 0 {
        crate::effects::add_mark_slot::add_mark_slot(
            get(settings + 0x64)?,
            MATRICES.view_place[0].addr(),
        );
    }
    crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[202].bits());
    for s in SLOTS.slots.iter().rev() {
        if (s.life.get() as i32) <= 0 {
            continue;
        }
        s.life
            .set(s.life.get().wrapping_sub(CLOCK.frame_ticks.get()));
        let p = (SPLINE_HEAD.points[0].addr()).wrapping_add(s.point.get().wrapping_mul(12));
        let [a, b, c] = [get(p)? ^ 0x8000_0000, get(p + 4)?, get(p + 8)?];
        let k4 = TUNING.place_k.each_ref().map(|f| f.bits());
        let z = x87::mul(c, k4[0]);
        let y = crate::math::x87_run::sub_mul(b, k4[1], k4[2]);
        let x = x87::mul(a, k4[3]);
        let once = rotate_point(&At(s.matrix.get()), [extended(z), extended(y), extended(x)]);
        let twice = rotate_point(&MATRICES.view_matrix, once);
        let [z, y, x] = twice.map(rounded);
        for (n, v) in [z, y, x].into_iter().enumerate() {
            SLOTS.work[n].set(v);
        }
        let r = TUNING
            .draw_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        TUNING.draw_random.set(r);
        let corner = &MARK_CORNERS.corners[((r & 7) + 0x50) as usize];
        let word = |off: u32| u32::from(corner.words[off as usize / 2].get());
        let uv_k: u32 = DRAW_WORK.uv_k.bits();
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            vertex.u.set_bits(x87::int_times(word(u) as i32, uv_k));
            vertex.v.set_bits(x87::int_times(word(v) as i32, uv_k));
        }
        for n in 0..4 {
            VERTICES.vertices[n].colour.set(0xFFFF_FFFFu32);
        }
        if (x as i32) <= (NUMBERS.k4096_a.bits() as i32) {
            continue;
        }
        let [px, py, half] = x87::slot_project(
            DRAW.view_scale.bits(),
            x,
            z,
            DRAW.view_centre[0].bits(),
            y,
            DRAW.view_centre[1].bits(),
            SLOTS.slots[0].size.bits(),
        );
        SLOTS.work[0].set(px);
        SLOTS.work[1].set(py);
        TUNING.half.set_bits(half);
        let corners = [
            (x87::sub(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::sub(py, half)),
            (x87::add(px, half), x87::add(py, half)),
            (x87::sub(px, half), x87::add(py, half)),
        ];
        let depth = x87::mul(x, DRAW_WORK.z_k.bits());
        let rhw = x87::reciprocal_times(x, TUNING.rhw_k.bits());
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
