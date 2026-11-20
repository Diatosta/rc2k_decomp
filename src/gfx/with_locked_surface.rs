//! The screen's surface locked for the duration of a closure: the one way the front end draws on
//! it, so every path out (an early `?` included) unlocks it.

use crate::errors::Refused;

/// The screen's surface while it is locked: unlocks it when dropped (drop it before showing
/// the frame).
pub struct LockedSurface<'a>(&'a crate::gfx::owner::Gfx);

impl Drop for LockedSurface<'_> {
    fn drop(&mut self) {
        crate::platform::unlock_surface(self.0);
    }
}

/// Locks the surface: `None` when it could not be locked.
pub fn lock<'a>(
    gfx: &'a crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Option<LockedSurface<'a>> {
    (crate::platform::lock_surface(gfx, system) >= 1).then_some(LockedSurface(gfx))
}

/// Runs `draw` with the surface locked (`locked`, from `lock`) and unlocks it afterwards.
/// `Ok(None)` when the surface could not be locked (`draw` is not run); otherwise what `draw`
/// returned.
pub fn with_locked_surface<T>(
    locked: Option<LockedSurface<'_>>,
    draw: impl FnOnce() -> Result<T, Refused>,
) -> Result<Option<T>, Refused> {
    let Some(_locked) = locked else {
        return Ok(None);
    };
    draw().map(Some)
}
