//! `scene::full_view_window` (0x44570A): `scene::choose_view_window` with `word_636526` 2
//! (800 x 600 when framed), the word back to 0 after.
//!
//! In:  nothing
//! Out: as `choose_view_window` leaves them
//!
//! Callers: `sub_45A893` (0x45A906), code at 0x4435B8.

use crate::scene::choose_view_window::choose_view_window;
use crate::scene::owner::scene;

/// `choose_view_window` with the mode 2, the mode back to 0 after.
pub fn full_view_window(race: &crate::race::owner::Race) {
    let light_tables = &scene().light_tables;
    light_tables.view_mode.set(2);
    choose_view_window(race);
    light_tables.view_mode.set(0);
}
