//! `scene::draw_menu_object`: draws the current object the way the menus show theirs.
//!
//! Sets the shine to 0x200, makes the matrix from the angles, place and size the caller set
//! (`scene::make_object_matrix`), sets `LightState.view_kind` = 5 and the light kind = 4, draws
//! (`scene::draw_scene_object`), then sets the shine to 0x100. 14 callers: the carousel's entry
//! and thirteen menus' objects.

use crate::errors::Refused;
use crate::scene::matrix::make_object_matrix;

// ---- game data ----

/// The two calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Matrix,
    Draw,
}

/// `step` stands for the two calls.
pub fn draw_menu_object(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
    light: &crate::scene::object_light::ObjectLight,
    mine: &crate::race::view_course::ViewCourse,
) -> Result<(), Refused> {
    {
        mine.shine.set(0x200u32);
        step(Step::Matrix)?;
        light.view_kind.set(5u32);
        light.light_kind.set(4u32);
        step(Step::Draw)?;
        mine.shine.set(0x100u32);
        Ok(())
    }
}

/// The object's description is read only in the scene's modes 1 and 2; the menus draw in
/// mode 0 (each caller writes it).
pub fn draw_menu_object_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
    let race = ctx.race;
    draw_menu_object(
        &mut |step| match step {
            Step::Matrix => {
                make_object_matrix(
                    &ctx.reads.buffers.engine_tables,
                    &ctx.view.matrices,
                    &race.frame.matrices.get().view_turn_matrix,
                    &ctx.view.light,
                    &ctx.view.object,
                );
                Ok(())
            }
            Step::Draw => crate::scene::draw_scene_object::draw_scene_object_in_game(ctx, None),
        },
        &ctx.view.light,
        &ctx.view.course,
    )
}
