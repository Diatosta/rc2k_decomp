//! `scene::prepare_menu_scene` (0x46F7FD): the menus' camera and light.
//!
//! In:  nothing
//! Out: as `scene::set_view_matrices` leaves them (eax = ebx = ecx = edx = 0, esi, edi,
//!      ebp three matrices)
//!
//! The view's place and its three angles = 0; the window 800 x 600 (`dword_62408C`,
//! `dword_624090`); `scene::set_view_window`; the light's lowest 0x50, highest 0xFF, level 0xB4
//! and strength 0xB4, its direction's angles `dword_635094` = 0x6E and `dword_635098` = 0x1E;
//! `scene::set_view_matrices`. 29 callers (the screen loops' set-up).

use crate::framework::game_write;
use crate::scene::draw_store::DRAW;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::set_view_window::set_view_window;
use crate::scene::view_matrix::{VIEW_ANGLES_RVA, VIEW_PLACE_RVA, set_view_matrices};

// ---- game data ----

// ---- Rust version ----

pub unsafe fn prepare_menu_scene() {
    unsafe {
        game_write(VIEW_PLACE_RVA, [0u32; 3]);
        game_write(VIEW_ANGLES_RVA, [0u32; 3]);
        crate::session::write_bits(&DRAW.view_size, [0x320u32, 0x258]);
        set_view_window();
        let o = &OBJECT_WORK;
        o.light_lowest.set(0x50);
        o.light_highest.set(0xFF);
        o.light_level.set(0xB4);
        o.light_strength.set(0xB4);
        crate::session::write_bits(&OBJECT_WORK.light_turns, [0x6Eu32, 0x1E]);
        set_view_matrices();
    }
}
