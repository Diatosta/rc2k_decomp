//! `scene::full_view_window` (0x44570A): `scene::choose_view_window` with `word_636526` 2
//! (800 x 600 when framed), the word back to 0 after.
//!
//! In:  nothing
//! Out: as `choose_view_window` leaves them
//!
//! Callers: `sub_45A893` (0x45A906), code at 0x4435B8.

use crate::framework::game_write;
use crate::scene::choose_view_window::{MODE_RVA, choose_view_window};

/// `choose_view_window` with the mode 2, the mode back to 0 after.
pub unsafe fn full_view_window() {
    unsafe {
        game_write(MODE_RVA, 2u16);
        choose_view_window();
        game_write(MODE_RVA, 0u16);
    }
}
