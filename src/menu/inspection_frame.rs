//! `menu::inspection_frame`: one frame of the inspection area after its first
//! (`menu::inspection_area_loop`'s later frame callback): `menu::frame_callback`'s steps in
//! another order and without its notice: lock the surface; the background
//! (`copy_picture_to_surface`); the draw list; unlock; the scene state 4; begin the 3D scene;
//! the scene list; end the scene; lock again; `draw_menu`; the overlay list; unlock; show the
//! frame. A failed lock, scene beginning or showing ends it there (a frame not made).
//!
//! The surface and the showing are the platform's, as in `frame_callback`.

use crate::errors::Refused;
use crate::gfx::with_locked_surface::with_locked_surface;
use crate::menu::frame_callback::{copy_picture, draw_menu, run_listed};
use crate::owners::Owners;
use crate::shell::frame::show_frame_now;

/// Whether the frame of `menu` was made.
pub(super) fn inspection_frame(
    o: Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
) -> Result<bool, Refused> {
    let first = with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            copy_picture(o.gfx, o.system)?;
            run_listed(o, &o.menus.screens.flow.draw_list)
        },
    )?;
    if first.is_none() {
        return Ok(false);
    }
    crate::scene::begin_scene::begin_scene(o.race, &o.gfx.d3d, o.scene.scratch.game_scratch());
    run_listed(o, &o.menus.screens.flow.scene_list)?;
    crate::scene::end_scene::end_scene(&o.gfx.d3d);
    let second = with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            draw_menu(o, menu)?;
            run_listed(o, &o.menus.screens.flow.overlay_list)
        },
    )?;
    if second.is_none() {
        return Ok(false);
    }
    Ok(show_frame_now(o))
}
