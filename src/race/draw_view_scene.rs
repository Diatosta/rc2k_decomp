//! Draws a view's 3D scene. With `view_covered` set (the player of this view is out of a two-player
//! race, `race::drop_absent_player`) only `race::draw_view_tint` runs. Else `view_cleared` and
//! `ObjectWork.view_cleared[1]` are 0 and in turn, with the race's sound tick between the groups:
//! the car lights' modes, the point records cleared, the fog, the camera's view, the sky and the
//! draw batch flushed, the view's light; the lamps reset, `view_lamps` 0, the course's view
//! prepared, its visible faces, the EAX environment, the lamps gathered and wanted; the sort
//! entries sorted and drawn, the race's slots; the cars' shadows, the skidmarks, the car lights;
//! the depth entries sorted and drawn, the batch flushed, the view's value stored.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::view_scratch::{DrawCtx, ViewScratch};

/// What a view's drawing hands on, none for a covered view (`draw_view_tint` only, which leaves the
/// last ones): its EAX weight (`race::eax_environment`), its value for its block
/// (`race::store_view_value`: the block and the sun on the screen) and whether its camera set its
/// view matrices (`race::camera_view`, handed on by `race::hand_on_view_matrices`). Applied by the
/// game's thread once the view is drawn, in the views' order (the drawing parts write none).
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewOutput {
    pub environment: Option<u32>,
    pub view_value: Option<(usize, u32)>,
    pub view_matrices: bool,
}

impl ViewOutput {
    /// The view's EAX weight as the race's environment (`audio::store::ENVIRONMENT`), where the
    /// audio reads it.
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

/// On `view`'s own working state; `filter`, the texture filter's switches it starts from
/// (`race::view_scratch::filter_input`, taken on the game's thread).
#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's owners, its view, scratch and reads, as `DrawCtx::new` takes them"
)]
pub fn draw_view_scene(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
    input: &crate::input::owner::Input,
    view: &ViewScratch,
    filter: [u32; 3],
    part: &crate::workers::PartScratch,
    reads: crate::workers::DrawReads<'_>,
    helper: Option<crate::workers::Helper<'_>>,
) -> Result<ViewOutput, Refused> {
    use crate::race::frame_draw::*;
    view.begin();
    let ctx = DrawCtx::new(race, scene, net, input, view, part, reads);
    let _held = view.matrices.hold(reads.matrices, part);
    // The shine is the stage's, as the frame's first view has it; each later view used to get the
    // previous one's last car's (fixed: "Each view lit by what the view before it left" in
    // `wiki/Fixed-bugs.md`).
    view.course.shine.set(reads.stage_prefs.shine_from.get());
    let input = filter;
    view.light.car_light.set(view.car.light_colour.get());
    view.polygon.smooth.set(input[0]);
    view.polygon.plain_first.set(input[1]);
    view.polygon.plain_second.set(input[2]);
    let lists = &view.lists;
    if view.car.view_covered.get() != 0 {
        draw_view_tint(&view.polygon, part, &view.face, reads, reads.screen, view)?;
        return Ok(ViewOutput::default());
    }
    // `view_cleared`, ObjectWork.view_cleared[1].
    for cell in &view.object.view_cleared {
        cell.set(0);
    }
    car_light_modes(
        &view.cars,
        reads.work,
        reads.vars,
        reads.view,
        view.polygon.view_mirrored.get() != 0,
        view.drawn_view(race),
    )?;
    clear_point_records(reads, part)?;
    set_up_fog(
        &reads.buffers.engine_tables,
        reads.stage_prefs,
        &view.fog,
        &view.polygon,
        &view.car,
        part,
    )?;
    camera_view(reads, &view.matrices, view)?;
    draw_sky(&ctx)?;
    view_light(&ctx, reads.has_mmx)?;
    reset_lamps(&ctx)?;
    // `view_lamps` is not set to 0 here: nothing reads it.
    prepare_course_view(reads, view, part)?;
    gather_visible_faces(&ctx)?;
    let environment = eax_environment(reads.course(), reads.work, &part.points, &view.grid)?;
    gather_lamps(&ctx)?;
    lamps_wanted(reads.work, reads.view, &view.lamps)?;
    // From here the view's own car can be drawn aside (`race::car_part`): its drawing reads nothing
    // the rest of the view's drawing makes.
    let rest = |car: Option<&crate::race::car_part::CarAside<'_, '_>>| -> Result<(), Refused> {
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
        draw_entries_around(&ctx, &lists.depth_entry_count, |n| lists.depth_from(n), car)?;
        crate::debug::timing::stop(crate::debug::timing::Stage::DepthEntries, begun);
        Ok(())
    };
    let car = match (helper, crate::race::car_part::own_car(&ctx)) {
        (Some(helper), Some(car))
            if crate::race::car_part::filter_cleared(
                &ctx,
                lists.sort_entries(),
                lists.sort_entry_count.get(),
            ) =>
        {
            Some((helper, car))
        }
        _ => None,
    };
    match car {
        Some((helper, car)) => {
            crate::race::car_part::draw_with_car_aside(helper, &ctx, car, |car| rest(Some(car)))?;
        }
        None => rest(None)?,
    }
    let view_value = view_value(reads.view, &view.polygon.sun, view.drawn_view(race))?;
    Ok(ViewOutput {
        environment: Some(environment),
        view_value: Some(view_value),
        view_matrices: true,
    })
}
