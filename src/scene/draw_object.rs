//! `scene::draw_object` (0x45045D): draws one object, in seven steps.
//!
//! In:  nothing (`g_objectNumber` the object)
//! Out: the registers as the seven steps leave them, the last being
//!      `scene::draw_object_faces`
//!
//! eax = `g_objectNumber`; `scene::set_object_light`; `scene::set_object_material`;
//! `scene::set_object_fade` (which may make `g_objectNumber` another object); ebx =
//! `g_objectNumber`; `scene::place_object_points_any` (0x45BDE1, inlined: with `g_use3dNow`
//! `sub_45BF89`, else with `g_useSse` `sub_45C04E`, neither ported, else
//! `scene::place_object_points`); `scene::list_object_faces`; `scene::sort_listed_faces`;
//! `scene::draw_object_faces`. One caller, `sub_4503A5`, which reads no register after it. The
//! Rust versions of the light, the material, the fade and the sorting keep the registers their
//! originals change, so what is left differs from the original's there. A refusal of a later
//! step comes with the earlier ones done; the original then does them again.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::scene::draw_object_face::draw_object_face_in_game;
use crate::scene::draw_object_faces::draw_object_faces;
use crate::scene::list_object_face::list_object_face;
use crate::scene::list_object_faces::list_object_faces;
use crate::scene::owner::scene;
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

/// `scene::place_object_points_any` (0x45BDE1). The 3DNow way (`sub_45BF89`) is not ported:
/// a refusal ends the program.
fn place_object_points_any(race: &crate::race::owner::Race) -> Result<(), Refused> {
    let draw_switches = &scene().draw_switches;
    if draw_switches.use_3dnow.bits() != 0 {
        return Err(BadInput("the 3DNow way of placing points is not ported").into());
    }
    if draw_switches.use_sse.bits() != 0 {
        return crate::scene::place_object_points::place_object_points_sse(race);
    }
    place_object_points(race)?;
    Ok(())
}

/// With the steps' Rust versions.
pub fn draw_object_in_game(race: &crate::race::owner::Race) -> Result<(), Refused> {
    draw_object(&mut |step| {
        match step {
            Step::Light => set_object_light_in_game(race)?,
            Step::Material => set_object_material(race)?,
            Step::Fade => set_object_fade(race)?,
            Step::Points => place_object_points_any(race)?,
            Step::List => {
                list_object_faces(&race.shared, &mut |face| list_object_face(race, face))?;
            }
            Step::Sort => sort_listed_faces()?,
            Step::Faces => {
                draw_object_faces(race, &mut |face| draw_object_face_in_game(race, face))?;
            }
        }
        Ok(())
    })
}
