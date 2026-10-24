//! `menu::inspection_screen` (0x4A1BBF): the inspection's screen shown: `dword_623FB0` /
//! `dword_623FB4` 0 and `dword_624080` 0x100, the menu scene prepared
//! (`scene::prepare_menu_scene`), the clip's screen edges 0 and 0x320 (`dword_6241DC` /
//! `dword_6241E0` and, as floats, `g_clipScreenLeft` / `Right`), the background loaded
//! (`service::load_background`, 0x470F55; its carry not read), copied to the surface and
//! the frame shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`.

use crate::errors::Refused;
use crate::framework::{game_write, rva};

const ZEROED_RVA: [usize; 2] = [rva(0x623FB0), rva(0x623FB4)]; // dword_623FB0, dword_623FB4
const SCALE_RVA: usize = rva(0x624080); // dword_624080
const EDGES_RVA: [usize; 2] = [rva(0x6241DC), rva(0x6241E0)]; // dword_6241DC, dword_6241E0
const CLIP_RVA: [usize; 2] = [rva(0x6241EC), rva(0x6241F0)]; // g_clipScreenLeft, g_clipScreenRight

/// The Rust version (see the header). Whether the copy and the showing worked is not
/// looked at (the original's carries are not).
pub unsafe fn inspection_screen() -> Result<(), Refused> {
    unsafe {
        game_write(ZEROED_RVA[0], 0u32);
        game_write(ZEROED_RVA[1], 0u32);
        game_write(SCALE_RVA, 0x100u32);
        crate::scene::prepare_menu_scene::prepare_menu_scene();
        game_write(EDGES_RVA[0], 0u32);
        game_write(EDGES_RVA[1], 0x320u32);
        game_write(CLIP_RVA[0], 0.0f32.to_bits());
        game_write(CLIP_RVA[1], 800.0f32.to_bits());
        crate::service::load_background::load_background()?;
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )?;
    }
    crate::platform::show_frame_now();
    Ok(())
}
