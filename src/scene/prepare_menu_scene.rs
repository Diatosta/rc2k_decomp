//! `scene::prepare_menu_scene` (0x46F7FD): the menus' camera and light.
//!
//! In:  nothing
//! Out: as `scene::set_view_matrices` leaves them (eax = ebx = ecx = edx = 0, esi, edi,
//!      ebp three matrices)
//!
//! The view's place and its three angles = 0; the window 800 x 600 (`dword_62408C`,
//! `dword_624090`); `scene::set_view_window`; the light's lowest 0x50, highest 0xFF,
//! level 0xB4 and strength 0xB4, its direction's angles `dword_635094` = 0x6E and
//! `dword_635098` = 0x1E; `scene::set_view_matrices`.
//! 29 callers (the screen loops' set-up).
//! Self-test: 20 settings by lot on the game's variables.

use crate::framework::{game_write, rva};
use crate::scene::set_view_window::{SIZE_RVA, set_view_window};
use crate::scene::view_matrix::{VIEW_ANGLES_RVA, VIEW_PLACE_RVA, set_view_matrices};
use crate::screens::harness::Left;

// ---- game data ----

const LIGHT_RVA: usize = rva(0x635084); // g_lightLowest, g_lightHighest, g_lightLevel, g_lightStrength
const LIGHT_ANGLES_RVA: usize = rva(0x635094); // dword_635094, dword_635098

// ---- Rust version ----

pub unsafe fn prepare_menu_scene() -> Left {
    unsafe {
        game_write(VIEW_PLACE_RVA, [0u32; 3]);
        game_write(VIEW_ANGLES_RVA, [0u32; 3]);
        game_write(SIZE_RVA, [0x320u32, 0x258]);
        set_view_window();
        game_write(LIGHT_RVA, [0x50u32, 0xFF, 0xB4, 0xB4]);
        game_write(LIGHT_ANGLES_RVA, [0x6Eu32, 0x1E]);
        set_view_matrices()
    }
}
