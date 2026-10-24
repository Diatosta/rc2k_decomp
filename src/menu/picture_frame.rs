//! `menu::picture_frame` (0x46F3FA): a frame of a still picture: the surface locked, the picture copied to it, unlocked, the frame shown.
//!
//! In:  nothing
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//!
//! The frame callback of `sub_47F314` and `sub_47F3C7` (handed to `screens::run_frame`). The steps are the game's
//! (the surface, the 3D scene and the showing are the platform's), through an `Env`
//! (`picture_frame` and `fade_frame` alike). A private copy (a call at +1). Shadow mode runs the original
//! alone (a frame cannot be shown twice). Self-test: none left.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;

/// The Rust version: whether the frame was made (the original's carry clear): the surface
/// locked (else no frame), the picture copied onto it, unlocked, shown.
pub fn picture_frame() -> Result<bool, Refused> {
    if crate::platform::lock_surface() < 1 {
        return Ok(false);
    }
    unsafe {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    crate::platform::unlock_surface();
    Ok(crate::platform::show_frame_now())
}
