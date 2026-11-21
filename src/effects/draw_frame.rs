//! `effects::draw_frame`: the effects' drawing frame. Three `points` turned by the view
//! matrix (`scene::rotate_point`) into `turned_points`; the course's corners (0x8000 bytes
//! of the course info, at its tail end) into the mark corners table; when the marks' source
//! switch changed since the last frame (`last_source`) the heading `step` steps on (of 8);
//! the sound source cell gets it, the sound zero cell 0, `cleared` 0; then
//! `effects::cars_particles`, `effects::view_heading`, `effects::view_motion`,
//! `effects::begin_quads` (its time, a cycle count, in `quads_share`), an empty time in
//! `empty_share`, and `scene::depth_test_on`.
//!
//! Refuses (before anything) a null course info, and `cars_particles`' refusals (what it
//! wrote before is put back first).

use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::rotate_point::rotate_point_by;
use core::cell::Cell;
const CORNERS_FROM: u32 = 0x1C_2000;
const CORNERS_LEN: u32 = 0x8000;
/// The two (time kept, where it goes) pairs.
fn times(effects: &Effects) -> [(f32, &Cell<f32>); 2] {
    let kept = effects.frame.draw_frame.borrow().times;
    [
        (kept[0], &effects.setup.frame.quads_share),
        (kept[1], &effects.setup.frame.empty_share),
    ]
}

/// Draws the frame: `cars_particles`, `view_heading`, `view_motion`, `begin_quads`,
/// `scene::depth_test_on`.
pub fn draw_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    effects: &Effects,
) -> Result<(), Refused> {
    use crate::effects::{begin_quads, cars_particles, view_heading, view_motion};
    if !crate::race::course_places::info_held(&race.course.stage.course) {
        return Err(BadInput("no course info").into());
    }
    let m = race.frame.view_kept.borrow().view_place;
    for n in 0..3 {
        let p: [u32; 3] = effects.frame.draw_frame.borrow().points[n].map(f32::to_bits);
        let turned = rotate_point_by(&m, [extended(p[2]), extended(p[1]), extended(p[0])]);
        for (k, v) in turned.iter().enumerate() {
            effects.frame.view_vars.borrow_mut().turned_points[n][k] = rounded_value(*v);
        }
    }
    let corners = crate::race::course_places::info_bytes(
        &race.course.stage.course,
        CORNERS_FROM,
        CORNERS_LEN as usize,
    )?;
    // 0x80 bytes of 16-bit little-endian words for each corner record.
    let (records, _) = corners.as_chunks::<0x80>();
    for (record, bytes) in effects.blocks.corners.corners.iter().zip(records) {
        let (pairs, _) = bytes.as_chunks::<2>();
        for (word, pair) in record.words.iter().zip(pairs) {
            word.set(u16::from_le_bytes(*pair));
        }
    }
    let source: u32 = scene.draw_switches().marks_source;
    if source != effects.frame.draw_frame.borrow().last_source {
        let step = effects.frame.view_vars.borrow().step.wrapping_add(1) & 7;
        effects.frame.view_vars.borrow_mut().step = step;
    }
    effects.frame.draw_frame.borrow_mut().last_source = source;
    effects.setup.work.borrow_mut().sound_source = source;
    effects.setup.work.borrow_mut().sound_zero = 0;
    effects.frame.view_vars.borrow_mut().cleared = 0;
    cars_particles::cars_particles(effects)?;
    view_heading::view_heading(&race.frame, effects);
    view_motion::view_motion(&race.frame, effects)?;
    let times = times(effects);
    let start = crate::effects::cycles::start();
    begin_quads::begin_quads(buffer, effects, scene);
    let frame = effects.setup.frame.frame_cycles.get();
    if let Some(took) = crate::effects::cycles::since(start) {
        times[0]
            .1
            .set(x87::int_over_int_times(took, frame, times[0].0));
    }
    // Nothing runs between these two readings: the page's "empty" time is the counter's own cost.
    let start = crate::effects::cycles::start();
    if let Some(took) = crate::effects::cycles::since(start) {
        times[1]
            .1
            .set(x87::int_over_int_times(took, frame, times[1].0));
    }
    crate::scene::depth_test_on::depth_test_on(scene.scratch.game_scratch());
    Ok(())
}
