//! `menu::show_loading_picture` (0x47CEA8): shows the picture "loading".
//!
//! `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.
//! Fifteen callers, `menu::set_up_front_end` among them.

use crate::gfx::show_picture_file::show_picture_file;
use crate::menu::owner::Menus;

// ---- game data ----

/// The picture shown.
pub fn show_loading_picture_in(
    menus: &Menus,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), crate::errors::Refused> {
    show_picture_file(&menus.tables.texts.loading_pcx, gfx, system)
}
