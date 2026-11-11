//! `menu::inspection_screen` (0x4A1BBF): the inspection's screen shown: `dword_623FB0` /
//! `dword_623FB4` 0 and `dword_624080` 0x100, the menu scene prepared
//! (`scene::prepare_menu_scene`), the clip's screen edges 0 and 0x320 (`dword_6241DC` /
//! `dword_6241E0` and, as floats, `g_clipScreenLeft` / `Right`), the background loaded
//! (`service::load_background`, 0x470F55; its carry not read), copied to the surface and
//! the frame shown.

use crate::errors::Refused;

/// Whether the copy and the showing worked is not looked at (the original's carries are
/// not).
pub fn inspection_screen() -> Result<(), Refused> {
    let scene = crate::scene::owner::scene();
    let front_end = crate::scene::polygon_work::front_end();
    front_end.window.view_offset[0].set_bits(0u32);
    front_end.window.view_offset[1].set_bits(0u32);
    front_end.window.view_zoom.set_bits(0x100u32);
    crate::scene::prepare_menu_scene::prepare_menu_scene(scene);
    front_end.window.clip_edges[2].set_bits(0u32);
    front_end.window.clip_edges[3].set_bits(0x320u32);
    front_end.window.clip_screen[2].set_bits(0.0f32.to_bits());
    front_end.window.clip_screen[3].set_bits(800.0f32.to_bits());
    crate::service::load_background::load_background()?;
    crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
        &mut crate::gfx::copy_picture_to_surface::GameSurface,
    )?;
    crate::platform::show_frame_now();
    Ok(())
}
