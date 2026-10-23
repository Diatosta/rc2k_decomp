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

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
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

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const USE_3DNOW_RVA: usize = rva(0x621A5C); // g_use3dNow
const USE_SSE_RVA: usize = rva(0x621A58); // g_useSse
const PLACE_3DNOW_RVA: usize = rva(0x45BF89); // sub_45BF89
const PLACE_SSE_RVA: usize = rva(0x45C04E); // sub_45C04E

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

/// `step`: what stands for each call, with the registers it is entered with.
pub unsafe fn draw_object(
    r: &mut Regs,
    step: &mut dyn FnMut(Step, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        r.eax = game_read(OBJECT_RVA);
        step(Step::Light, r)?;
        step(Step::Material, r)?;
        step(Step::Fade, r)?;
        r.ebx = game_read(OBJECT_RVA);
        step(Step::Points, r)?;
        step(Step::List, r)?;
        step(Step::Sort, r)?;
        step(Step::Faces, r)
    }
}

/// `scene::place_object_points_any` (0x45BDE1).
unsafe fn place_object_points_any(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(USE_3DNOW_RVA) != 0 {
            crate::calls::call(PLACE_3DNOW_RVA, r);
        } else if game_read::<u32>(USE_SSE_RVA) != 0 {
            crate::calls::call(PLACE_SSE_RVA, r);
        } else {
            place_object_points(r)?.apply(r);
        }
        Ok(())
    }
}

/// With the steps' Rust versions.
pub unsafe fn draw_object_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        draw_object(r, &mut |step, r| {
            match step {
                Step::Light => set_object_light_in_game()?,
                Step::Material => set_object_material()?,
                Step::Fade => set_object_fade()?,
                Step::Points => place_object_points_any(r)?,
                Step::List => list_object_faces(&mut |face| list_object_face(face))?.apply(r),
                Step::Sort => sort_listed_faces()?,
                Step::Faces => {
                    let entry = *r;
                    draw_object_faces(&mut |face| draw_object_face_in_game(face, &entry))?.apply(r)
                }
            }
            Ok(())
        })
    }
}
