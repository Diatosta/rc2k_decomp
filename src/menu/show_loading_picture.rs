//! `menu::show_loading_picture`: shows the picture "loading".
//!
//! `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.

use crate::gfx::show_picture_file::show_picture_file;

/// The picture shown.
pub fn show_loading_picture_in(
    o: crate::owners::Owners<'static>,
) -> Result<(), crate::errors::Refused> {
    show_picture_file(o, &o.menus.texts.loading_pcx)
}
