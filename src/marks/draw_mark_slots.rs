//! `marks::draw_mark_slots` (0x4C969A): the mark slots drawn, one square sprite each. When
//! the settings' +0x8C is set, `marks::add_mark_slot` first (eax the settings' +0x64, esi
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
//! Caller: `marks::run_frame` (0x4D1659). Refuses (before anything) null settings and a
//! live slot with a null matrix.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::mark_slots::{SLOTS, TUNING};
use crate::marks::setup::CLOCK;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

const PLACE_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: add_mark_slot's esi
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const POINTS_RVA: Slot = Slot::at(0x769312); // flt_769312: 12 bytes each
const VIEW_RVA: usize = rva(0x5EA2CC); // g_viewMatrix
const CORNERS_RVA: Slot = Slot::at(0x8EB971); // g_marksCorners
const UV_K_RVA: Slot = Slot::at(0x8EB969); // flt_8EB969
const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const Z_K_RVA: Slot = Slot::at(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4]
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Why it refuses, if it does, worked out before anything is written.
unsafe fn refusal() -> Option<&'static str> {
    {
        if crate::marks::setup::SETTINGS.pointer.get() == 0 {
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
pub unsafe fn draw_mark_slots() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if get(settings + 0x8C) != 0 {
            crate::marks::add_mark_slot::add_mark_slot(
                get(settings + 0x64),
                game_addr(PLACE_RVA) as u32,
            );
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        for s in SLOTS.slots.iter().rev() {
            if (s.life.get() as i32) <= 0 {
                continue;
            }
            s.life
                .set(s.life.get().wrapping_sub(CLOCK.frame_ticks.get()));
            let p = (game_addr(POINTS_RVA) as u32).wrapping_add(s.point.get().wrapping_mul(12));
            let [a, b, c] = [get(p) ^ 0x8000_0000, get(p + 4), get(p + 8)];
            let k4 = TUNING.place_k.each_ref().map(|f| f.bits());
            let z = x87::mul(c, k4[0]);
            let y = crate::math::x87_run::sub_mul(b, k4[1], k4[2]);
            let x = x87::mul(a, k4[3]);
            let once = rotate_point(s.matrix.get(), [extended(z), extended(y), extended(x)]);
            let twice = rotate_point(game_addr(VIEW_RVA) as u32, once);
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
            let corner = game_addr(CORNERS_RVA) as u32 + ((r & 7) + 0x50) * 0x80;
            let word = |off: u32| ((corner + off) as *const u16).read_unaligned() as u32;
            let uv_k: u32 = game_read(UV_K_RVA);
            for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
                let vertex = VERTICES_RVA + 0x20 * n;
                game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
                game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
            }
            for n in 0..4 {
                game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
            }
            if (x as i32) <= (NUMBERS.k4096_a.bits() as i32) {
                continue;
            }
            let [px, py, half] = x87::slot_project(
                DRAW.view_scale.bits(),
                x,
                z,
                game_read(CENTRE_RVA[0]),
                y,
                game_read(CENTRE_RVA[1]),
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
            let depth = x87::mul(x, game_read(Z_K_RVA));
            let rhw = x87::reciprocal_times(x, TUNING.rhw_k.bits());
            for (n, (vx, vy)) in corners.into_iter().enumerate() {
                let vertex = VERTICES_RVA + 0x20 * n;
                game_write(vertex, vx);
                game_write(vertex + 4, vy);
                game_write(vertex + 8, depth);
                game_write(vertex + 0xC, rhw);
            }
            crate::marks::clip_quad::draw_fan();
        }
    }
    Ok(())
}
