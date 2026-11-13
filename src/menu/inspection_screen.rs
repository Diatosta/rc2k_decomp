//! `menu::inspection_screen`: the inspection's screen shown: two scene work cells set to 0 and
//! 0x100, the menu scene prepared (`scene::prepare_menu_scene`), the clip's screen edges 0 and
//! 0x320 (as scene work cells and, as floats, the clip screen left / right), the background
//! loaded (`service::load_background`; its result not read), copied to the surface and the
//! frame shown.

use crate::errors::Refused;

/// Whether the copy and the showing worked is not looked at.
pub fn inspection_screen(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let scene = o.scene;
    let front_end = crate::scene::polygon_work::front_end();
    front_end.window.view_offset[0].set(0);
    front_end.window.view_offset[1].set(0);
    front_end.window.view_zoom.set(0x100);
    crate::scene::prepare_menu_scene::prepare_menu_scene(
        o.race,
        scene,
        o.system,
        crate::race::view_scratch::front_end(),
    );
    front_end.window.clip_edges[2].set(0);
    front_end.window.clip_edges[3].set(0x320);
    front_end.window.clip_screen[2].set(0.0);
    front_end.window.clip_screen[3].set(800.0);
    crate::service::load_background::load_background_in(o.menus, o.gfx, o.system)?;
    crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
        &mut crate::gfx::copy_picture_to_surface::GameSurface(o.gfx, o.system),
    )?;
    crate::platform::show_frame_now(o);
    Ok(())
}
