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
//! Caller: `marks::run_frame` (0x4D1659). Its calls through `part` (1 `add_mark_slot`, 2
//! `set_texture`, 3 `draw_fan`); in the game their Rust versions (the device's through
//! the game). Refuses (before anything) null settings and a live slot with a null matrix.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose three calls
//! go to stand-ins (the turns real; each draw notes the four vertices), against the Rust
//! version on the same, on settings, slots, points and matrices of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::setup::{SETTINGS_RVA, SLOT_LEN, SLOTS_RVA};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The slots it goes through: 0x1F down to 0 (`push 20h`, then `dec` before the first).
const DRAWN: u32 = 0x20;
const PLACE_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: add_mark_slot's esi
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const POINTS_RVA: usize = rva(0x769312); // flt_769312: 12 bytes each
const WORK_RVA: usize = rva(0x9352F4); // dword_9352F4 .. dword_9352FC
/// flt_9353EE, flt_9353F2, flt_9353F6, flt_9353FA.
const PLACE_K: [usize; 4] = [rva(0x9353EE), rva(0x9353F2), rva(0x9353F6), rva(0x9353FA)];
const VIEW_RVA: usize = rva(0x5EA2CC); // g_viewMatrix
const RANDOM_RVA: usize = rva(0x9353FE); // dword_9353FE
const CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const NEAR_RVA: usize = rva(0x5C5E18); // flt_5C5E18
const FOCAL_RVA: usize = rva(0x624088); // flt_624088
const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const SIZE_K_RVA: usize = rva(0x934F8C); // flt_934F8C: slot 0's +0x10
const HALF_RVA: usize = rva(0x9353A6); // flt_9353A6
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const RHW_K_RVA: usize = rva(0x935402); // flt_935402
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4]
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

fn slot(k: u32) -> u32 {
    game_addr(SLOTS_RVA) as u32 + k * SLOT_LEN
}

/// Why it refuses, if it does, worked out before anything is written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Some("no marks settings");
        }
        for k in 0..DRAWN {
            if (get(slot(k)) as i32) > 0 && get(slot(k) + 8) == 0 {
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
        let settings: u32 = game_read(SETTINGS_RVA);
        if get(settings + 0x8C) != 0 {
            crate::marks::add_mark_slot::add_mark_slot(
                get(settings + 0x64),
                game_addr(PLACE_RVA) as u32,
            );
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        for k in (0..DRAWN).rev() {
            let s = slot(k);
            if (get(s) as i32) <= 0 {
                continue;
            }
            (s as *mut u32).write_unaligned(get(s).wrapping_sub(game_read(TICKS_RVA)));
            let p = (game_addr(POINTS_RVA) as u32).wrapping_add(get(s + 4).wrapping_mul(12));
            let [a, b, c] = [get(p) ^ 0x8000_0000, get(p + 4), get(p + 8)];
            let k4 = PLACE_K.map(|r| game_read::<u32>(r));
            let z = x87::mul(c, k4[0]);
            let y = crate::math::x87_run::sub_mul(b, k4[1], k4[2]);
            let x = x87::mul(a, k4[3]);
            let once = rotate_point(get(s + 8), [extended(z), extended(y), extended(x)]);
            let twice = rotate_point(game_addr(VIEW_RVA) as u32, once);
            let [z, y, x] = twice.map(rounded);
            for (n, v) in [z, y, x].into_iter().enumerate() {
                game_write(WORK_RVA + 4 * n, v);
            }
            let r = game_read::<u32>(RANDOM_RVA)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            game_write(RANDOM_RVA, r);
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
            if (x as i32) <= game_read::<i32>(NEAR_RVA) {
                continue;
            }
            let [px, py, half] = x87::slot_project(
                game_read(FOCAL_RVA),
                x,
                z,
                game_read(CENTRE_RVA[0]),
                y,
                game_read(CENTRE_RVA[1]),
                game_read(SIZE_K_RVA),
            );
            game_write(WORK_RVA, px);
            game_write(WORK_RVA + 4, py);
            game_write(HALF_RVA, half);
            let corners = [
                (x87::sub(px, half), x87::sub(py, half)),
                (x87::add(px, half), x87::sub(py, half)),
                (x87::add(px, half), x87::add(py, half)),
                (x87::sub(px, half), x87::add(py, half)),
            ];
            let depth = x87::mul(x, game_read(Z_K_RVA));
            let rhw = x87::reciprocal_times(x, game_read(RHW_K_RVA));
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
