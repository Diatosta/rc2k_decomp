//! `menu::show_loading_picture`: shows the picture "loading".
//!
//! `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.
//! Fifteen callers, `menu::set_up_front_end` among them.

use crate::gfx::show_picture_file::show_picture_file;
use crate::menu::texts_store::texts;

/// The picture shown.
pub fn show_loading_picture_in(
    o: crate::owners::Owners<'static>,
) -> Result<(), crate::errors::Refused> {
    show_picture_file(o, &texts().loading_pcx)
}
