//! `race::draw_view_scene` (0x4557AB): a view's 3D scene. With `view_covered` set (`view_covered`: the player of this view is out of a two-player
//! race, `race::drop_absent_player`) only `race::draw_view_tint`. Else `view_cleared`, `ObjectWork.view_cleared[1]` 0 and in turn,
//! with the race's sound tick between the groups: the car lights' modes, the point records cleared,
//! the fog, the camera's view, the sky and the draw batch flushed, the view's light; the lamps
//! reset, `view_lamps` 0, the course's view prepared, its visible faces, the EAX environment, the
//! lamps gathered and wanted; the sort entries sorted and drawn, the race's slots; the cars'
//! shadows, the skidmarks, the car lights; the depth entries sorted and drawn, the batch flushed,
//! the view's value stored.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::view_scratch::{DrawCtx, ViewScratch};

/// What a view's drawing hands on (frame plan F6), none for a covered view
/// (`draw_view_tint` only, which leaves the last ones): its EAX weight
/// (`race::eax_environment`), its value for its block (`race::store_view_value`: the
/// block and the sun on the screen) and whether its camera set its view matrices
/// (`race::camera_view`, handed on by `race::hand_on_view_matrices`). Applied by the game's
/// thread once the view is drawn, in the views' order (the drawing parts write none).
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewOutput {
    pub environment: Option<u32>,
    pub view_value: Option<(usize, u32)>,
    pub view_matrices: bool,
}

impl ViewOutput {
    /// The view's EAX weight as the race's environment (`audio::store::ENVIRONMENT`), where
    /// the audio reads it (the original made its sum there in place, the audio thread
    /// reading it at any time).
    pub fn publish_environment(self) {
        if let Some(weight) = self.environment {
            crate::audio::store::ENVIRONMENT.set(weight);
        }
    }

    /// The view's value into its block (`race::store_view_value`).
    pub fn store_view_value(self, race: &Race) -> Result<(), Refused> {
        match self.view_value {
            Some((block, value)) => crate::race::frame_draw::store_view_value(race, block, value),
            None => Ok(()),
        }
    }
}

/// On `view`'s own working state.
pub fn draw_view_scene(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    net: &crate::net::owner::Net,
    input: &crate::input::owner::Input,
    view: &ViewScratch,
    part: &crate::workers::PartScratch,
) -> Result<ViewOutput, Refused> {
    use crate::race::frame_draw::*;
    view.begin();
    let ctx = DrawCtx::new(race, scene, system, net, input, view, part);
    let _held = view.matrices.hold(&race.frame.matrices, part);
    // The shine from the stage's, as the frame's first view has it: the original left each
    // later view the previous one's last car's (a slip fixed: `wiki/Fixed-bugs.md`).
    view.course
        .shine
        .set(race.course.stage_prefs.shine_from.get());
    let input = crate::race::view_scratch::filter_input(view);
    view.light.car_light.set(view.car.light_colour.get());
    view.polygon.smooth.set(input[0]);
    view.polygon.plain_first.set(input[1]);
    view.polygon.plain_second.set(input[2]);
    let lists = &view.lists;
    if view.car.view_covered.get() != 0 {
        draw_view_tint(
            &view.polygon,
            part,
            race,
            &view.face,
            scene,
            &system.session.screen,
            view,
        )?;
        return Ok(ViewOutput::default());
    }
    // `view_cleared`, ObjectWork.view_cleared[1].
    for cell in &view.object.view_cleared {
        cell.set(0);
    }
    car_light_modes(
        race,
        &view.cars,
        view.polygon.view_mirrored.get() != 0,
        view.drawn_view(race),
    )?;
    clear_point_records(scene, part)?;
    set_up_fog(race, &view.fog, &view.polygon, &view.car, part)?;
    camera_view(race, scene, system, &view.matrices, view)?;
    draw_sky(&ctx)?;
    view_light(&ctx, crate::race::frame_draw::has_mmx(system))?;
    reset_lamps(&ctx)?;
    // (The original's 0 into `g_viewLamps`, 0x63DB6C: read by nothing.)
    prepare_course_view(race, scene, view, part)?;
    gather_visible_faces(&ctx)?;
    let environment = eax_environment(race, &part.points, &view.grid)?;
    gather_lamps(&ctx)?;
    lamps_wanted(&race.shared, &view.lamps)?;
    sort_entries(&lists.sort_entry_count, lists.sort_entries())?;
    let begun = crate::debug::timing::start();
    draw_entries(&ctx, &lists.sort_entry_count, |n| lists.sort_from(n))?;
    crate::debug::timing::stop(crate::debug::timing::Stage::CourseEntries, begun);
    draw_race_slots(&ctx)?;
    crate::race::car_shadow::cast_car_shadows(&ctx)?;
    crate::race::skidmarks::draw_skidmarks_if_on(&ctx)?;
    car_lights(&ctx)?;
    sort_entries(&lists.depth_entry_count, lists.depth_entries())?;
    let begun = crate::debug::timing::start();
    draw_entries(&ctx, &lists.depth_entry_count, |n| lists.depth_from(n))?;
    crate::debug::timing::stop(crate::debug::timing::Stage::DepthEntries, begun);
    let view_value = view_value(race, &view.polygon.sun, view.drawn_view(race))?;
    // The lamps the next view starts from (`race::lamp_work::PUBLISHED`).
    {
        use crate::cells::Cells;
        crate::race::lamp_work::PUBLISHED.copy_from(&view.lamps);
    }
    Ok(ViewOutput {
        environment: Some(environment),
        view_value: Some(view_value),
        view_matrices: true,
    })
}
