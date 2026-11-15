//! Frees the installer's memory list and zeroes its folder- and file-name table pointers.
//! Called by `install::run_installer`.

use crate::errors::Refused;

pub fn close_installer(
    install: &crate::install::owner::Install,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    { crate::gfx::glyph_store::free_list(&gfx.fonts, &install.run.buffers) }?;
    install.run.dirs.set(0);
    install.run.files.set(0);
    install.folders.give_back();
    install.files.give_back();
    Ok(())
}
