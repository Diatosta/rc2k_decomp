//! `scene::draw_object` (0x45045D): draws one object, in seven steps.
//!
//! In:  nothing (`g_objectNumber` the object)
//! Out: the registers as the seven steps leave them, the last being
//!      `scene::draw_object_faces`
//!
//! eax = `g_objectNumber`; `scene::set_object_light`; `scene::set_object_material`;
//! `scene::set_object_fade` (which may make `g_objectNumber` another object);
//! ebx = `g_objectNumber`; `scene::place_object_points_any` (0x45BDE1, inlined: with
//! `g_use3dNow` `sub_45BF89`, else with `g_useSse` `sub_45C04E`, neither ported, else
//! `scene::place_object_points`); `scene::list_object_faces`;
//! `scene::sort_listed_faces`; `scene::draw_object_faces`.
//! One caller, `sub_4503A5`, which reads no register after it.
//! The Rust versions of the light, the material, the fade and the sorting keep the
//! registers their originals change, so what is left differs from the original's there.
//! A refusal of a later step comes with the earlier ones done; the original then does
//! them again.
//! Shadow mode runs the original alone (each step is compared in its own hook).
//! Self-test: a redirected copy whose seven calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::framework::{game_read, rva};
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

const USE_3DNOW_RVA: usize = rva(0x621A5C); // g_use3dNow
const USE_SSE_RVA: usize = rva(0x621A58); // g_useSse

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
pub unsafe fn draw_object(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
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

/// `scene::place_object_points_any` (0x45BDE1). The 3DNow and SSE ways (`sub_45BF89`,
/// `sub_45C04E`) are not ported: a call of either ended the program ("not ported"), a
/// refusal ends it now.
unsafe fn place_object_points_any() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(USE_3DNOW_RVA) != 0 || game_read::<u32>(USE_SSE_RVA) != 0 {
            return Err(BadInput("the 3DNow and SSE ways of placing points are not ported").into());
        }
        place_object_points()?;
        Ok(())
    }
}

/// With the steps' Rust versions.
pub unsafe fn draw_object_in_game() -> Result<(), Refused> {
    unsafe {
        draw_object(&mut |step| {
            match step {
                Step::Light => set_object_light_in_game()?,
                Step::Material => set_object_material()?,
                Step::Fade => set_object_fade()?,
                Step::Points => place_object_points_any()?,
                Step::List => {
                    list_object_faces(&mut |face| list_object_face(face))?;
                }
                Step::Sort => sort_listed_faces()?,
                Step::Faces => {
                    draw_object_faces(&mut |face| draw_object_face_in_game(face))?;
                }
            }
            Ok(())
        })
    }
}
