//! `menu::frame_callback` (0x474D4B): one frame of a menu screen, the callback nearly every
//! screen loop hands to `screens::run_frame`.
//!
//! In this order: lock the surface; `copy_picture_to_surface` (the background); `draw_menu`; every
//! function of the draw list (`g_drawList`); unlock; the scene state = 4; begin the 3D scene
//! (`scene::begin_scene`); every function of the scene list (`g_sceneList`); end the scene
//! (`scene::end_scene`); lock again; every function of the overlay list (`g_overlayList`); if
//! the window-active flag is 2 and `cheats.notice_due` is 1, clear that and draw string 0x1000 at (800, 587) with
//! the font 0x51E404; unlock; show the frame (0x4057D1). A set carry after either lock, the scene's
//! beginning or the showing ends it there, with what was locked left locked. The functions of the
//! draw and overlay lists get the registers as the one before left them; a list ends with -1. Rust
//! callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::fn_list::ListCell;
use crate::owners::Owners;
use crate::platform::{lock_surface, show_frame_now, unlock_surface};
use crate::text::draw_text::GameCanvas;
use rc2k_core::menu::notice;

const NOTICE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NOTICE;

/// Runs the functions of the list `list` points at (the original keeps the registers
/// around each of the scene list's; its functions take none either way).
pub fn run_listed(o: Owners<'static>, list: &ListCell) -> Result<(), Refused> {
    crate::menu::run_list::run_list(o, list.get()?)
}

/// `gfx::lock_surface`'s carry: the surface could not be locked.
pub fn lock_failed(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) -> bool {
    lock_surface(gfx, system) < 1
}

/// `gfx::copy_picture_to_surface` (0x4060E3): the background.
pub fn copy_picture(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface(gfx, system),
        )
    }?;
    Ok(())
}

/// `menu::draw_menu` (0x4746B7) on the game's canvas.
pub fn draw_menu(
    o: Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
) -> Result<(), Refused> {
    crate::menu::draw_menu::draw_menu(o, menu, &mut GameCanvas::new(o.gfx))
}

/// `menu` is edi at entry. Returns whether the
/// frame was made (the original's clear carry).
pub fn frame_callback(
    o: Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let play = &menus.play;
    if lock_failed(o.gfx, o.system) {
        return Ok(false);
    }
    copy_picture(o.gfx, o.system)?;
    draw_menu(o, menu)?;
    run_listed(o, &o.menus.screens.flow.draw_list)?;
    unlock_surface(o.gfx);
    o.scene.lights.state.scene_state.set(4);
    crate::scene::begin_scene::begin_scene(o.race, o.d3d, crate::workers::game_scratch());
    run_listed(o, &o.menus.screens.flow.scene_list)?;
    crate::scene::end_scene::end_scene(o.d3d);
    if lock_failed(o.gfx, o.system) {
        return Ok(false);
    }
    run_listed(o, &o.menus.screens.flow.overlay_list)?;
    // The logic is in `rc2k_core::menu::notice::drawn_now`.
    if notice::drawn_now(
        crate::strings::load_language::notice_kind(o.strings),
        || play.cheats.notice_due.get(),
    ) {
        play.cheats.notice_due.set(0u32);
        crate::text::draw_string_by_id::draw_string_by_id_at(
            crate::strings::append_string_by_id::StringOwners::of(o),
            notice::STRING,
            NOTICE_FONT,
            notice::AT.0 as i32,
            notice::AT.1 as i32,
            &mut GameCanvas::new(o.gfx),
        )?;
    }
    unlock_surface(o.gfx);
    Ok(show_frame_now())
}
