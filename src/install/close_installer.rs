//! Frees the installer's memory list and gives back its folder- and file-name tables.

use crate::errors::Refused;

pub(super) fn close_installer(
    install: &crate::install::owner::Install,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    { crate::gfx::glyph_store::free_list(&gfx.fonts, &install.run.buffers) }?;
    install.folders.give_back();
    install.files.give_back();
    Ok(())
}
