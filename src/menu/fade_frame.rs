//! `menu::fade_frame` (0x4A1E60): a frame of the inspection's fades: the picture copied to the locked surface, then in a 3D scene the fade's see-through quad (`screens::draw_fade_overlay`), the frame shown.
//!
//! In:  nothing
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//!
//! The frame callback of `sub_4A1CFA` and `sub_4A1DA5` (handed to `screens::run_frame`). The steps are the game's
//! (the surface, the 3D scene and the showing are the platform's), through an `Env`
//! (`picture_frame` and `fade_frame` alike). A private copy (a call at +1). Shadow mode runs the original
//! alone (a frame cannot be shown twice). Self-test: none left.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;

/// The Rust version: whether the frame was made (the original's carry clear): as
/// `picture_frame`, with the fade's overlay drawn in a scene between the unlock and the
/// showing (`scene::begin_scene` never sets the carry).
pub fn fade_frame() -> Result<bool, Refused> {
    if crate::platform::lock_surface() < 1 {
        return Ok(false);
    }
    unsafe {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    crate::platform::unlock_surface();
    crate::scene::begin_scene::begin_scene();
    crate::screens::draw_fade_overlay::draw_fade_overlay()?;
    crate::scene::end_scene::end_scene();
    Ok(crate::platform::show_frame_now())
}
