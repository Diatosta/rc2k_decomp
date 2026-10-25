//! `scene::choose_view_window` (0x44FE1A): the view's window, then `scene::set_view_window`:
//! without `dword_621514` the screen's size (`g_screenWidth`, `g_screenHeight`) at 0, 0;
//! with it 800 x 600 at 0, 0 when `dword_621480` is set or `word_636526` is 2, else 720 x
//! 540 at 0x50, 0; `dword_624080` 0x100 either way.
//!
//! In:  nothing
//! Out: as `set_view_window` leaves them
//!
//! Callers: `scene::full_view_window`, code at 0x446810. `set_view_window` is its Rust version.

use crate::framework::{game_read, rva};
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::draw_store::DRAW;
use crate::session::SCREEN;
pub const MODE_RVA: usize = rva(0x636526); // word_636526

/// The window chosen, then `set_view_window`.
pub unsafe fn choose_view_window() {
    // Converted: `rc2k_core::scene::view_window::choose`.
    use rc2k_core::scene::view_window::{ViewSettings, choose};
    unsafe {
        let settings = ViewSettings {
            framed: FRAME_SWITCHES.editor_mode.bits() != 0,
            wide: FRAME_SWITCHES.other_way.bits() != 0,
            mode: game_read(MODE_RVA),
        };
        let window = choose(settings, || [SCREEN.width.get(), SCREEN.height.get()]);
        DRAW.view_size[0].set_bits(window.size[0]);
        DRAW.view_size[1].set_bits(window.size[1]);
        DRAW.view_offset[0].set_bits(window.offset[0]);
        DRAW.view_offset[1].set_bits(window.offset[1]);
        DRAW.view_zoom.set_bits(window.scale);
        crate::scene::set_view_window::set_view_window();
    }
}
