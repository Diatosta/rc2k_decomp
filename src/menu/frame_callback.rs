//! `menu::frame_callback`: one frame of a menu screen, the callback nearly every screen loop
//! hands to `screens::run_frame`.
//!
//! In this order: lock the surface; `copy_picture_to_surface` (the background); `draw_menu`; every
//! function of the draw list; unlock; the scene state = 4; begin the 3D scene
//! (`scene::begin_scene`); every function of the scene list; end the scene (`scene::end_scene`);
//! lock again; every function of the overlay list; if the window-active flag is 2 and
//! `cheats.notice_due` is 1, clear that and draw string 0x1000 at (800, 587) in the notice font;
//! unlock; show the frame. A failed lock, scene beginning or showing ends it there. A list ends with -1.

use crate::errors::Refused;
use crate::gfx::with_locked_surface::with_locked_surface;
use crate::menu::fn_list::ListCell;
use crate::owners::Owners;
use crate::shell::frame::show_frame_now;
use crate::text::draw_text::GameCanvas;
use rc2k_core::menu::notice;

const NOTICE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NOTICE;

/// Runs the functions of the list `list` points at.
pub fn run_listed(o: Owners<'static>, list: &ListCell) -> Result<(), Refused> {
    crate::menu::run_list::run_list(o, list.get()?)
}

/// `gfx::copy_picture_to_surface`: the background.
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

/// `menu::draw_menu` on the game's canvas.
pub fn draw_menu(
    o: Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
) -> Result<(), Refused> {
    crate::menu::draw_menu::draw_menu(o, menu, &mut GameCanvas::new(o.gfx))
}

/// Draws one frame of `menu`. Returns whether the frame was made.
pub fn frame_callback(
    o: Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let play = &menus.play;
    let first = with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            copy_picture(o.gfx, o.system)?;
            draw_menu(o, menu)?;
            run_listed(o, &o.menus.screens.flow.draw_list)
        },
    )?;
    if first.is_none() {
        return Ok(false);
    }
    o.scene.lights.state.get().scene_state.set(4);
    crate::scene::begin_scene::begin_scene(o.race, &o.gfx.d3d, o.scene.scratch.game_scratch());
    run_listed(o, &o.menus.screens.flow.scene_list)?;
    crate::scene::end_scene::end_scene(&o.gfx.d3d);
    let second = with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            run_listed(o, &o.menus.screens.flow.overlay_list)?;
            if notice::drawn_now(crate::strings::locale::notice_kind(o.strings), || {
                play.cheats.notice_due.get()
            }) {
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
            Ok(())
        },
    )?;
    if second.is_none() {
        return Ok(false);
    }
    Ok(show_frame_now(o))
}
