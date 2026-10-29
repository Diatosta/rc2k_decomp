//! `screens::service_area_frame` (0x46CFD7): one frame of the service area (the callback
//! `screens::run_frame` is given): the back surface locked (`gfx::lock_surface`), the
//! picture copied in (`gfx::copy_picture_to_surface`), unlocked; `scene::begin_scene`,
//! `screens::draw_service_boxes`, `scene::end_scene`; locked again, the text and icons
//! (`screens::draw_service_area`), unlocked; `gfx::show_frame`. A lock, the scene's begin
//! or the show failing: carry set.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set on a failure, else clear with ZF
//! from the show's eax
//!
//! Callers: through the frame callbacks of `service::apply_repairs` and its kind (stored at
//! 0x46C33E .. 0x46CF7C).

use crate::errors::Refused;
use crate::framework::Flags;
use crate::platform::{lock_surface, show_frame_now, unlock_surface};

/// The Rust version (see the header): the flags it leaves (the registers are kept). The
/// lock's carry is `cmp g_surfaceLocked, 1`'s (its zero clear then). On a frame not shown
/// (quitting) the zero flag was what the call's way back left, nothing meaningful: clear
/// here. On the way through, the show's eax is the 0 the scene's begin left: zero set.
pub fn service_area_frame() -> Result<Flags, Refused> {
    use crate::screens::{draw_service_area, draw_service_boxes};
    const FAILED: Flags = Flags {
        carry: true,
        zero: false,
    };
    if lock_surface() < 1 {
        return Ok(FAILED);
    }
    crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
        &mut crate::gfx::copy_picture_to_surface::GameSurface,
    )?;
    unlock_surface();
    crate::scene::begin_scene::begin_scene();
    draw_service_boxes::draw_service_boxes()?;
    crate::scene::end_scene::end_scene();
    if lock_surface() < 1 {
        return Ok(FAILED);
    }
    draw_service_area::draw_service_area(&mut crate::text::draw_text::GameCanvas)?;
    unlock_surface();
    if !show_frame_now() {
        return Ok(FAILED);
    }
    Ok(Flags {
        carry: false,
        zero: true,
    })
}
