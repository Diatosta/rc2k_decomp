//! `race::draw_view_scene` (0x4557AB): a view's 3D scene. With `unk_71AF80` set (a reading:
//! the view covered) only `race::draw_view_tint`. Else `dword_635014`, `dword_635018` 0 and
//! in turn, with the race's sound tick (`sound::race_sound_tick`, eax 0x14 .. 0x1A in the
//! original: the Rust tick takes none) between the groups: the car lights' modes, the point
//! records cleared, the fog, the camera's view, the sky and the draw batch flushed, the
//! view's light; the lamps reset, `dword_63DB6C` 0, the course's view prepared, its visible
//! faces, the EAX environment, the lamps gathered and wanted; the sort entries sorted and
//! drawn, the race's slots; the cars' shadows, the skidmarks, the car lights; the depth
//! entries sorted and drawn, the batch flushed, the view's value stored.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::view_scratch::{DrawCtx, ViewScratch};

/// What a view's drawing hands on (frame plan F6): its EAX weight (`race::eax_environment`),
/// none for a covered view (`draw_view_tint` only, which leaves the last one).
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewOutput {
    pub environment: Option<u32>,
}

impl ViewOutput {
    /// The output where the audio reads it: the view's EAX weight as the race's environment
    /// (`audio::store::ENVIRONMENT`), at the view's end as the original left it (its sum was
    /// made there in place, the audio thread reading it at any time).
    pub fn publish(self) {
        if let Some(weight) = self.environment {
            crate::audio::store::ENVIRONMENT.set(weight);
        }
    }
}

/// The Rust version (see the header), on `view`'s own working state.
pub fn draw_view_scene(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    view: &ViewScratch,
) -> Result<ViewOutput, Refused> {
    use crate::race::frame_draw::*;
    view.begin();
    let ctx = DrawCtx::new(race, scene, view);
    let _held = view.matrices.hold(&race.frame.matrices);
    // The shine from the stage's, as the frame's first view has it: the original left each
    // later view the previous one's last car's (a slip fixed: `wiki/Fixed-bugs.md`).
    view.course
        .shine
        .set(race.course.stage_prefs.shine_from.get());
    let input = crate::race::view_scratch::filter_input(view);
    view.light
        .car_light
        .set(crate::race::frame::drawn_car().light_colour.bits());
    view.polygon.smooth.set(input[0]);
    view.polygon.plain_first.set(input[1]);
    view.polygon.plain_second.set(input[2]);
    let lists = &view.lists;
    if crate::race::frame::drawn_car().view_covered.get() != 0 {
        draw_view_tint(&view.polygon, race, &view.face, scene, view)?;
        return Ok(ViewOutput::default());
    }
    // dword_635014, dword_635018.
    for cell in &scene.drawing.object_work.view_cleared {
        cell.set(0);
    }
    car_light_modes(race)?;
    clear_point_records(race, scene)?;
    set_up_fog(race, scene, &view.fog, &view.polygon)?;
    camera_view(race, scene, &view.matrices)?;
    draw_sky(&ctx)?;
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    view_light(race, scene, &view.light)?;
    reset_lamps(&ctx)?;
    race.shared.view.view_lamps.set_bits(0u32);
    prepare_course_view(race, view)?;
    gather_visible_faces(&ctx)?;
    let environment = eax_environment(race, &view.grid)?;
    gather_lamps(&ctx)?;
    lamps_wanted(&race.shared)?;
    sort_entries(&lists.sort_entry_count, lists.sort_entries())?;
    let c = crate::moved::census::before();
    let begun = crate::debug::timing::start();
    draw_entries(&ctx, &lists.sort_entry_count, |n| lists.sort_from(n))?;
    crate::debug::timing::stop(crate::debug::timing::Stage::CourseEntries, begun);
    crate::moved::census::after("sorted", c);
    draw_race_slots(&ctx)?;
    crate::race::car_shadow::cast_car_shadows(&ctx)?;
    crate::race::skidmarks::draw_skidmarks_if_on(&ctx)?;
    car_lights(&ctx)?;
    sort_entries(&lists.depth_entry_count, lists.depth_entries())?;
    let c = crate::moved::census::before();
    let begun = crate::debug::timing::start();
    draw_entries(&ctx, &lists.depth_entry_count, |n| lists.depth_from(n))?;
    crate::debug::timing::stop(crate::debug::timing::Stage::DepthEntries, begun);
    crate::moved::census::after("depth", c);
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    store_view_value(race)?;
    Ok(ViewOutput {
        environment: Some(environment),
    })
}
