//! `menu::frame_callback` (0x474D4B): one frame of a menu screen, the callback nearly every
//! screen loop hands to `screens::run_frame`.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: carry = the frame could not be made; every register preserved
//! In this order: lock the surface; `copy_picture_to_surface` (the background); `draw_menu`; every
//! function of the draw list (`g_drawList`); unlock; `dword_6365CC` = 4; begin the 3D scene
//! (`sub_44E8A1`); every function of the scene list (`g_sceneList`), each between `pusha` and
//! `popa`; end the scene (`sub_44E952`); lock again; every function of the overlay list
//! (`g_overlayList`); if `byte_4E07A0` is 2 and `dword_7585D8` is 1, clear that and draw string
//! 0x1000 at (800, 587) with the font 0x51E404; unlock; show the frame (`sub_4057D1`). A set carry
//! after either lock, the scene's beginning or the showing ends it there, with what was locked left
//! locked. The functions of the draw and overlay lists get the registers as the one before left
//! them; a list ends with -1. Rust callers run its `entry` (`routes.rs`); the registers above are
//! the original's, for reference.

use crate::errors::Refused;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::table_calls::call_listed;
use crate::platform::{lock_surface, show_frame_now, unlock_surface};
use crate::scene::light_store::LIGHT_STATE;
use crate::session::Dword;
use crate::text::draw_text::GameCanvas;
use rc2k_core::menu::notice;

// ---- game data ----

const DRAW_LIST: &Dword = &SCREEN_FLOW.draw_list; // g_drawList
const SCENE_LIST: &Dword = &SCREEN_FLOW.scene_list; // g_sceneList
const OVERLAY_LIST: &Dword = &SCREEN_FLOW.overlay_list; // g_overlayList
const NOTICE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NOTICE;
const END: u32 = u32::MAX;

// ---- Rust version ----

/// Runs the functions of the list `list` points at (the original keeps the registers
/// around each of the scene list's; its functions take none either way).
pub fn run_listed(list: &Dword) -> Result<(), Refused> {
    let mut at: u32 = list.get();
    while crate::pointers::GamePtr::new(at).read::<u32>("frame callback: a game value")? != END {
        call_listed(
            crate::pointers::GamePtr::new(at).read::<u32>("frame callback: a game value")?,
        )?;
        at = at.wrapping_add(4);
    }

    Ok(())
}

/// `gfx::lock_surface`'s carry: the surface could not be locked.
pub fn lock_failed() -> bool {
    lock_surface() < 1
}

/// `gfx::copy_picture_to_surface` (0x4060E3): the background.
pub fn copy_picture() -> Result<(), Refused> {
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    Ok(())
}

/// `menu::draw_menu` (0x4746B7) with `palette` and `measure_x` as ebx and ecx.
pub unsafe fn draw_menu(menu: u32, palette: u32, measure_x: u32) -> Result<(), Refused> {
    unsafe { crate::menu::draw_menu::draw_menu(menu, palette, measure_x, &mut GameCanvas) }
}

/// `menu`, `palette` and `measure_x` are edi, ebx and ecx at entry. Returns whether the
/// frame was made (the original's clear carry).
pub unsafe fn frame_callback(menu: u32, palette: u32, measure_x: u32) -> Result<bool, Refused> {
    unsafe {
        if lock_failed() {
            return Ok(false);
        }
        copy_picture()?;
        draw_menu(menu, palette, measure_x)?;
        run_listed(DRAW_LIST)?;
        unlock_surface();
        LIGHT_STATE.scene_state.set_bits(4u32);
        crate::scene::begin_scene::begin_scene();
        run_listed(SCENE_LIST)?;
        crate::scene::end_scene::end_scene();
        if lock_failed() {
            return Ok(false);
        }
        run_listed(OVERLAY_LIST)?;
        // Converted: `rc2k_core::menu::notice::drawn_now`.
        if notice::drawn_now(crate::strings::load_language::notice_kind(), || {
            CHEAT_VARS.notice_due.get()
        }) {
            CHEAT_VARS.notice_due.set(0u32);
            crate::text::draw_string_by_id::draw_string_by_id_at(
                notice::STRING,
                NOTICE_FONT.as_ptr(),
                notice::AT.0 as i32,
                notice::AT.1 as i32,
                &mut GameCanvas,
            )?;
        }
        unlock_surface();
        Ok(show_frame_now())
    }
}
