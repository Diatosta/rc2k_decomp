//! `race::viewer_snapshot`: saves the viewer's frame as a picture, the next numbered file in
//! `savedata\\snapshot\\` (the screenshot key's folder and numbering: one more than the highest
//! there, so no picture is written over), while the surface is locked (`gfx::lock_surface`; if
//! that fails, nothing is saved).

use crate::errors::Refused;

pub fn viewer_snapshot(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    // Only when the surface could be locked.
    crate::gfx::with_locked_surface::with_locked_surface(
        crate::gfx::with_locked_surface::lock(gfx, system),
        || crate::gfx::write_screen_pcx::write_screen_pcx(None, true, gfx, system),
    )?;
    Ok(())
}
