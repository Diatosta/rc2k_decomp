//! `install::backdrop_frame` (0x415273, `loc_415273`, inside `install::show_backdrop`): the frame
//! of
//! `install::show_backdrop`: the surface locked (`gfx::lock_surface`; failing: carry set,
//! nothing more), the backdrop picture copied to it, unlocked and shown.
//!
//! Caller `screens::run_frame` as the frame callback.

use crate::errors::Refused;
use crate::install::installer::{copy_picture, lock, show, unlock};

/// The surface locked, the backdrop drawn, the frame shown; the carry it leaves (the
/// lock's or the show's).
pub fn backdrop_frame(gfx: &crate::gfx::owner::Gfx) -> Result<bool, Refused> {
    if !lock(gfx) {
        return Ok(true);
    }
    copy_picture()?;
    unlock(gfx);
    Ok(!show())
}
