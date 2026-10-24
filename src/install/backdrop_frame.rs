//! `install::backdrop_frame` (0x415273, `loc_415273`, inside `sub_41525D`): the frame of
//! `install::show_backdrop`: the surface locked (`gfx::lock_surface`; failing: carry set,
//! nothing more), the backdrop picture copied to it, unlocked and shown.
//!
//! In:  nothing
//! Out: as `gfx::show_frame` leaves them, or the lock's carry
//! Caller `screens::run_frame` (eax this address).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{copy_picture, lock, show, unlock};

/// The surface locked, the backdrop drawn, the frame shown; the carry it leaves (the
/// lock's or the show's).
pub fn backdrop_frame() -> Result<bool, Refused> {
    if !lock() {
        return Ok(true);
    }
    copy_picture()?;
    unlock();
    Ok(!show())
}
