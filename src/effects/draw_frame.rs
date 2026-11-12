//! `effects::draw_frame` (0x4C3EEE, through its tail): the effects' drawing frame. Three
//! `points` turned by the view matrix (`scene::rotate_point`) into `turned_points` (z, y, x of
//! each on the x87 stack as the original loads them); the course's corners (0x8000 bytes of the
//! course info, from +0x1C2000) into the mark corners table; when the marks' source switch
//! changed since the last frame (`last_source`) the heading `step` steps on (of 8); the sound
//! source cell gets it, the sound zero cell 0, `cleared` 0; then `effects::cars_particles`,
//! `effects::view_heading`, `effects::view_motion`, `effects::begin_quads` (its time, `rdtsc`,
//! in `quads_share`), an empty time in `empty_share`, and `scene::depth_test_on`.
//!
//! Refuses (before anything) a null course info, and `cars_particles`' refusals (what it
//! wrote before is put back first).

use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::{PlainFloat, read_bits};
const CORNERS_FROM: u32 = 0x1C_2000;
const CORNERS_LEN: u32 = 0x8000;
/// (the time kept, where it goes).
fn times(effects: &Effects) -> [(&PlainFloat, &PlainFloat); 2] {
    [
        (
            &effects.frame.draw_frame.times[0],
            &effects.setup.frame.quads_share,
        ),
        (
            &effects.frame.draw_frame.times[1],
            &effects.setup.frame.empty_share,
        ),
    ]
}

fn rdtsc() -> u32 {
    crate::arch::rdtsc() as u32
}

/// `cars_particles`, `view_heading`, `view_motion`,
/// `begin_quads`, `scene::depth_test_on`.
pub fn draw_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    effects: &Effects,
) -> Result<(), Refused> {
    use crate::effects::{begin_quads, cars_particles, view_heading, view_motion};
    let info: u32 = race.course.lists.course_info.get();
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
    let corners = crate::race::course_places::info_bytes(CORNERS_FROM, CORNERS_LEN as usize)?;
    // 0x80 bytes of 16-bit little-endian words for each corner record.
    let (records, _) = corners.as_chunks::<0x80>();
    for (record, bytes) in effects.blocks.corners.corners.iter().zip(records) {
        let (pairs, _) = bytes.as_chunks::<2>();
        for (word, pair) in record.words.iter().zip(pairs) {
            word.set(u16::from_le_bytes(*pair));
        }
    }
    let source: u32 = scene.draw_switches.marks_source.get();
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
    view_heading::view_heading(&race.frame, effects);
    view_motion::view_motion(&race.frame, effects)?;
    let times = times(effects);
    let start = rdtsc();
    begin_quads::begin_quads(buffer, effects, scene);
    let frame = effects.setup.frame.frame_cycles.bits() as i32;
    let took = rdtsc().wrapping_sub(start) as i32;
    times[0]
        .1
        .set(x87::int_over_int_times(took, frame, times[0].0));
    let start = rdtsc();
    let took = rdtsc().wrapping_sub(start) as i32;
    times[1]
        .1
        .set(x87::int_over_int_times(took, frame, times[1].0));
    crate::scene::depth_test_on::depth_test_on(crate::workers::game_scratch());
    Ok(())
}
