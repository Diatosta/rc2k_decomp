//! The frame of `install::show_backdrop`: locks the surface (nothing more if that fails), copies
//! the backdrop picture to it, unlocks it and shows it. `screens::run_frame` runs it as the frame
//! callback.

use crate::errors::Refused;
use crate::install::installer::{copy_picture, lock, show, unlock};

/// Draws and shows one frame of the backdrop; true when the lock or the show failed.
pub(super) fn backdrop_frame(o: crate::owners::Owners<'static>) -> Result<bool, Refused> {
    let (gfx, system) = (o.gfx, o.system);
    if !lock(gfx, system) {
        return Ok(true);
    }
    copy_picture(gfx, system)?;
    unlock(gfx);
    Ok(!show(o))
}
