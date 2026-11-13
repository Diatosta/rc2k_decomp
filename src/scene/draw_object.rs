//! `scene::draw_object` (0x45045D): draws one object, in seven steps.
//!
//! In:  nothing (the current object number is the object)
//!
//! The steps: `scene::set_object_light`; `scene::set_object_material`; `scene::set_object_fade`
//! (which may make the current object another object); `scene::place_object_points_any`
//! (inlined: with the 3DNow switch on its way, not ported; with SSE on its way; else
//! `scene::place_object_points`); `scene::list_object_faces`; `scene::sort_listed_faces`;
//! `scene::draw_object_faces`. One caller, `scene::draw_shown_object`. A refusal of a later step
//! comes with the earlier ones done; the original then does them again.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::scene::draw_object_face::draw_object_face_in_game;
use crate::scene::draw_object_faces::draw_object_faces;
use crate::scene::list_object_face::list_object_face;
use crate::scene::list_object_faces::list_object_faces;
use crate::scene::place_object_points::place_object_points;
use crate::scene::set_object_fade::set_object_fade;
use crate::scene::set_object_light::set_object_light_in_game;
use crate::scene::set_object_material::set_object_material;
use crate::scene::sort_listed_faces::sort_listed_faces;

// ---- game data ----

/// The seven calls, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Light,
    Material,
    Fade,
    Points,
    List,
    Sort,
    Faces,
}

// ---- Rust version ----

/// `step`: what stands for each call. The registers between the calls carry nothing
/// a later call reads (the object's number goes to each through `g_objectNumber`).
pub fn draw_object(step: &mut dyn FnMut(Step) -> Result<(), Refused>) -> Result<(), Refused> {
    for s in [
        Step::Light,
        Step::Material,
        Step::Fade,
        Step::Points,
        Step::List,
        Step::Sort,
        Step::Faces,
    ] {
        step(s)?;
    }
    Ok(())
}

/// `scene::place_object_points_any` (0x45BDE1). The 3DNow way (0x45BF89) is not ported:
/// a refusal ends the program.
fn place_object_points_any(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let draw_switches = ctx.reads.draw_switches;
    if draw_switches.use_3dnow.get() != 0 {
        return Err(BadInput("the 3DNow way of placing points is not ported").into());
    }
    if draw_switches.use_sse.is_set() {
        return crate::scene::place_object_points::place_object_points_sse(ctx);
    }
    place_object_points(ctx)?;
    Ok(())
}

/// With the steps' Rust versions.
pub fn draw_object_in_game(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    // The object's listed faces: filled, sorted and drawn here alone (the original's
    // `g_listedFaces`, one for every drawing).
    let mut faces = crate::scene::face_list::FaceList::new();
    draw_object(&mut |step| {
        match step {
            Step::Light => set_object_light_in_game(ctx)?,
            Step::Material => set_object_material(ctx)?,
            Step::Fade => set_object_fade(ctx)?,
            Step::Points => place_object_points_any(ctx)?,
            Step::List => {
                list_object_faces(
                    &ctx.reads.buffers.faces,
                    &ctx.view.object,
                    &ctx.view.face,
                    &mut faces,
                    &mut |list, face| list_object_face(ctx, face, list, ctx.marks),
                )?;
            }
            Step::Sort => sort_listed_faces(&mut faces)?,
            Step::Faces => {
                draw_object_faces(
                    &ctx.reads.buffers.faces,
                    &ctx.view.polygon,
                    &ctx.view.object,
                    &ctx.view.face,
                    &faces,
                    &mut |face| draw_object_face_in_game(ctx, face),
                )?;
            }
        }
        Ok(())
    })
}
