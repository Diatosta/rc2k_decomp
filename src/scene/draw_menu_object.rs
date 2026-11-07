//! `scene::draw_menu_object` (0x4907E7): draws the object `g_objectNumber` the way the
//! menus show theirs.
//!
//! In:  nothing
//! Out: eax = 0x100; the rest as `scene::draw_scene_object` leaves them
//!
//! `g_shine` = 0x200 (`scene::set_shine` 0x422292, inlined); `scene::make_object_matrix` from
//! the angles, place and size the caller set; `dword_6365C8` = 5; `g_lightKind` = 4;
//! `scene::draw_scene_object`; `g_shine` = 0x100. 14 callers: the carousel's entry and thirteen
//! menus' objects.

use crate::errors::Refused;
use crate::scene::matrix::make_object_matrix;
use crate::scene::owner::scene;

// ---- game data ----

/// The two calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Matrix,
    Draw,
}

// ---- Rust version ----

/// `step`: what stands for the two calls. The original's eax (0x200, then 0x100: the
/// shine) is read by neither call nor by its callers.
pub fn draw_menu_object(
    course: &crate::race::owner_course::Course,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let lights = &scene().lights;
    {
        course.course.shine.set_bits(0x200u32);
        step(Step::Matrix)?;
        lights.state.view_kind.set_bits(5u32);
        lights.work.light_kind.set_bits(4u32);
        step(Step::Draw)?;
        course.course.shine.set_bits(0x100u32);
        Ok(())
    }
}

/// With the Rust versions of what it calls. The drawing's esi (the description) is read
/// only in the scene's modes 1 and 2; the menus draw in mode 0 (each caller writes it).
pub fn draw_menu_object_in_game(race: &crate::race::owner::Race) -> Result<(), Refused> {
    draw_menu_object(&race.course, &mut |step| match step {
        Step::Matrix => {
            make_object_matrix(&race.frame);
            Ok(())
        }
        Step::Draw => crate::scene::draw_scene_object::draw_scene_object_in_game(race, None),
    })
}
