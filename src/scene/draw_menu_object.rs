//! `scene::draw_menu_object` (0x4907E7): draws the object `g_objectNumber` the way the
//! menus show theirs.
//!
//! In:  nothing
//! Out: eax = 0x100; the rest as `scene::draw_scene_object` leaves them
//!
//! `g_shine` = 0x200 (`scene::set_shine` 0x422292, inlined); `scene::make_object_matrix`
//! from the angles, place and size the caller set; `dword_6365C8` = 5; `g_lightKind`
//! = 4; `scene::draw_scene_object`; `g_shine` = 0x100.
//! 14 callers: the carousel's entry and thirteen menus' objects.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose two calls note what was set by then.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::scene::matrix::make_object_matrix;

// ---- game data ----

const SHINE_RVA: usize = rva(0x5CCE74); // g_shine
const VIEW_KIND_RVA: usize = rva(0x6365C8); // dword_6365C8
const LIGHT_KIND_RVA: usize = rva(0x62DAF4); // g_lightKind

/// The two calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Matrix,
    Draw,
}

// ---- Rust version ----

/// `step`: what stands for the two calls. The original's eax (0x200, then 0x100: the
/// shine) is read by neither call nor by its callers.
pub unsafe fn draw_menu_object(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        game_write(SHINE_RVA, 0x200u32);
        step(Step::Matrix)?;
        game_write(VIEW_KIND_RVA, 5u32);
        game_write(LIGHT_KIND_RVA, 4u32);
        step(Step::Draw)?;
        game_write(SHINE_RVA, 0x100u32);
        Ok(())
    }
}

/// With the Rust versions of what it calls. The drawing's esi (the description) is read
/// only in the scene's modes 1 and 2; the menus draw in mode 0 (each caller writes it).
pub unsafe fn draw_menu_object_in_game() -> Result<(), Refused> {
    unsafe {
        draw_menu_object(&mut |step| match step {
            Step::Matrix => {
                make_object_matrix();
                Ok(())
            }
            Step::Draw => crate::scene::draw_scene_object::draw_scene_object_in_game(0),
        })
    }
}
