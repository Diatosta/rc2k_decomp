//! `effects::draw_frame` (0x4C3EEE, through 0x4C4E28): the marks' drawing frame. Three
//! points (`flt_91FF4A` .., `flt_91FF56` .., `flt_91FF62` ..) turned by the view matrix
//! (`scene::rotate_point`) into `flt_8F39A9` .. (z, y, x of each on the x87 stack as the
//! original loads them); the course's corners (`g_courseInfo` + 0x1C2000, 0x8000 bytes)
//! into `g_marksCorners`; when `dword_621A80` changed since the last frame
//! (`dword_91FF6E`) `dword_8F3979` steps on (of 8); `dword_71C8E2` it, `dword_71C8E6` 0,
//! `dword_8F3981` 0; then `effects::cars_particles`, `effects::view_heading`,
//! `effects::view_motion`, `effects::begin_quads` (its time, `rdtsc`, in `flt_765124`), an
//! empty time in `flt_765118`, and `scene::depth_test_on`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Refuses (before anything) a null `g_courseInfo`, and `cars_particles`' refusals (what it
//! wrote before is put back first).

use crate::effects::blocks::MARK_CORNERS;
use crate::effects::frame_state::{DRAW_FRAME, VIEW_VARS};
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::effects::tuning::MARKS_FRAME;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::matrices_store::MATRICES;
use crate::scene::draw_switches_store::DRAW_SWITCHES;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::{Float, read_bits};
const CORNERS_FROM: u32 = 0x1C_2000;
const CORNERS_LEN: u32 = 0x8000;
/// (the time kept, where it goes).
static TIMES: [(&Float, &Float); 2] = [
    (&DRAW_FRAME.times[0], &MARKS_FRAME.quads_share),
    (&DRAW_FRAME.times[1], &MARKS_FRAME.empty_share),
];

fn rdtsc() -> u32 {
    crate::arch::rdtsc() as u32
}

/// The Rust version (see the header): `cars_particles`, `view_heading`, `view_motion`,
/// `begin_quads`, `scene::depth_test_on`.
pub fn draw_frame(particles: &Particles, ring: &Ring) -> Result<(), Refused> {
    use crate::effects::{begin_quads, cars_particles, view_heading, view_motion};
    let info: u32 = COURSE_LISTS.course_info.bits();
    if info == 0 {
        return Err(BadInput("no course info (g_courseInfo 0)").into());
    }
    let m = &MATRICES.view_place;
    for n in 0..3 {
        let p: [u32; 3] = read_bits(&DRAW_FRAME.points[n]);
        let turned = rotate_point(m, [extended(p[2]), extended(p[1]), extended(p[0])]);
        for (k, v) in turned.iter().enumerate() {
            VIEW_VARS.turned_points[n][k].set_bits(rounded(*v));
        }
    }
    let what = "a stage record's corners";
    let corners = crate::race::course_places::info_bytes(CORNERS_FROM, CORNERS_LEN as usize, what)?;
    GamePtr::to(&MARK_CORNERS.corners).write_bytes(&corners, what)?;
    let source: u32 = DRAW_SWITCHES.marks_source.bits();
    if source != DRAW_FRAME.last_source.get() {
        VIEW_VARS.step.set(VIEW_VARS.step.get().wrapping_add(1) & 7);
    }
    DRAW_FRAME.last_source.set(source);
    MARKS_WORK.sound_source.set(source);
    MARKS_WORK.sound_zero.set(0);
    VIEW_VARS.cleared.set(0);
    cars_particles::cars_particles(particles, ring)?;
    view_heading::view_heading();
    view_motion::view_motion()?;
    let start = rdtsc();
    begin_quads::begin_quads();
    let frame = MARKS_FRAME.frame_cycles.bits() as i32;
    let took = rdtsc().wrapping_sub(start) as i32;
    TIMES[0]
        .1
        .set_bits(x87::int_over_int_times(took, frame, TIMES[0].0.bits()));
    let start = rdtsc();
    let took = rdtsc().wrapping_sub(start) as i32;
    TIMES[1]
        .1
        .set_bits(x87::int_over_int_times(took, frame, TIMES[1].0.bits()));
    crate::scene::depth_test_on::depth_test_on();
    Ok(())
}
