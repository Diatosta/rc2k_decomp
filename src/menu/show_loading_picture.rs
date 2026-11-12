//! `menu::show_loading_picture` (0x47CEA8): shows the picture "loading".
//!
//! `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.
//! Fifteen callers, `menu::set_up_front_end` among them.

use crate::gfx::show_picture_file::show_picture_file;

// ---- game data ----

/// The picture shown.
pub fn show_loading_picture_in(
    o: crate::owners::Owners<'static>,
) -> Result<(), crate::errors::Refused> {
    show_picture_file(
        crate::strings::append_string_by_id::StringOwners::of(o),
        &o.menus.tables.texts.loading_pcx,
        o.gfx,
        o.system,
    )
}
