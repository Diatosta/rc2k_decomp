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

use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::{Float, read_bits};
const CORNERS_FROM: u32 = 0x1C_2000;
const CORNERS_LEN: u32 = 0x8000;
/// (the time kept, where it goes).
static TIMES: [(&Float, &Float); 2] = [
    (
        &crate::effects::owner::EFFECTS.frame.draw_frame.times[0],
        &crate::effects::owner::EFFECTS.setup.frame.quads_share,
    ),
    (
        &crate::effects::owner::EFFECTS.frame.draw_frame.times[1],
        &crate::effects::owner::EFFECTS.setup.frame.empty_share,
    ),
];

fn rdtsc() -> u32 {
    crate::arch::rdtsc() as u32
}

/// The Rust version (see the header): `cars_particles`, `view_heading`, `view_motion`,
/// `begin_quads`, `scene::depth_test_on`.
pub fn draw_frame(race: &Race, effects: &Effects) -> Result<(), Refused> {
    use crate::effects::{begin_quads, cars_particles, view_heading, view_motion};
    let info: u32 = race.course.lists.course_info.bits();
    if info == 0 {
        return Err(BadInput("no course info (g_courseInfo 0)").into());
    }
    let m = &race.frame.matrices.view_place;
    for n in 0..3 {
        let p: [u32; 3] = read_bits(&effects.frame.draw_frame.points[n]);
        let turned = rotate_point(m, [extended(p[2]), extended(p[1]), extended(p[0])]);
        for (k, v) in turned.iter().enumerate() {
            effects.frame.view_vars.turned_points[n][k].set_bits(rounded(*v));
        }
    }
    let what = "a stage record's corners";
    let corners = crate::race::course_places::info_bytes(CORNERS_FROM, CORNERS_LEN as usize, what)?;
    // 0x80 bytes of 16-bit little-endian words for each corner record.
    let (records, _) = corners.as_chunks::<0x80>();
    for (record, bytes) in effects.blocks.corners.corners.iter().zip(records) {
        let (pairs, _) = bytes.as_chunks::<2>();
        for (word, pair) in record.words.iter().zip(pairs) {
            word.set(u16::from_le_bytes(*pair));
        }
    }
    let source: u32 = crate::scene::owner::scene()
        .draw_switches
        .marks_source
        .bits();
    if source != effects.frame.draw_frame.last_source.get() {
        effects
            .frame
            .view_vars
            .step
            .set(effects.frame.view_vars.step.get().wrapping_add(1) & 7);
    }
    effects.frame.draw_frame.last_source.set(source);
    effects.setup.work.sound_source.set(source);
    effects.setup.work.sound_zero.set(0);
    effects.frame.view_vars.cleared.set(0);
    cars_particles::cars_particles(effects)?;
    view_heading::view_heading(&race.frame);
    view_motion::view_motion(&race.frame)?;
    let start = rdtsc();
    begin_quads::begin_quads();
    let frame = effects.setup.frame.frame_cycles.bits() as i32;
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
