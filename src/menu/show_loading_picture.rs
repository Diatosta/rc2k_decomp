//! `menu::show_loading_picture` (0x47CEA8): shows the picture "loading".
//!
//! In:  nothing
//! Out: as `gfx::show_picture_file` leaves the registers (eax = 1, or the path if the
//! file could not be read) `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.
//! Fifteen callers, `menu::set_up_front_end` among them.

use crate::gfx::show_picture_file::show_picture_file;
use crate::menu::owner::menus;

// ---- game data ----

/// The picture shown.
pub fn show_loading_picture() -> Result<(), crate::errors::Refused> {
    show_picture_file(crate::pointers::GamePtr::to(
        &menus().tables.texts.loading_pcx,
    ))
}
