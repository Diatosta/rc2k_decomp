//! `menu::inspection_frame` (0x46D019): one frame of the inspection area after its first
//! (`menu::inspection_area_loop`'s later frame callback): `menu::frame_callback`'s steps in
//! another order and without its notice: lock the surface; the background
//! (`copy_picture_to_surface`); the draw list (`g_drawList`); unlock; `dword_6365CC` 4;
//! begin the 3D scene; the scene list (`g_sceneList`, each between `pusha` and `popa`);
//! end the scene; lock again; `draw_menu`; the overlay list (`g_overlayList`); unlock;
//! show the frame. A set carry after either lock, the scene's beginning or the showing ends
//! it there (carry set), with what was locked left locked; else the carry is cleared.
//!
//! In:  edi = the menu
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//! The draw and overlay lists' functions get the registers as the one before left them.
//! The surface and the showing are the platform's, as in `frame_callback`.

use crate::errors::Refused;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::fn_list::ListCell;
use crate::menu::frame_callback::{copy_picture, draw_menu, lock_failed, run_listed};
use crate::platform::{show_frame_now, unlock_surface};
use crate::scene::light_store::LIGHT_STATE;

const DRAW_LIST: &ListCell = &SCREEN_FLOW.draw_list; // g_drawList
const SCENE_LIST: &ListCell = &SCREEN_FLOW.scene_list; // g_sceneList
const OVERLAY_LIST: &ListCell = &SCREEN_FLOW.overlay_list; // g_overlayList

/// The Rust version, `menu` being edi at entry: whether the frame was made.
pub fn inspection_frame(menu: crate::menu::menus_store::MenuId) -> Result<bool, Refused> {
    if lock_failed() {
        return Ok(false);
    }
    copy_picture()?;
    run_listed(DRAW_LIST)?;
    unlock_surface();
    LIGHT_STATE.scene_state.set_bits(4u32);
    crate::scene::begin_scene::begin_scene();
    run_listed(SCENE_LIST)?;
    crate::scene::end_scene::end_scene();
    if lock_failed() {
        return Ok(false);
    }
    draw_menu(menu)?;
    run_listed(OVERLAY_LIST)?;
    unlock_surface();
    Ok(show_frame_now())
}
