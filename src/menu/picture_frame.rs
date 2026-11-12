//! `menu::picture_frame` (0x46F3FA): a frame of a still picture: the surface locked, the picture copied to it, unlocked, the frame shown.
//!
//! The frame callback of `menu::championship_welcome` and `menu::a8_welcome` (handed to
//! `screens::run_frame`). The steps are the game's (the surface, the 3D scene and the showing are
//! the platform's), through an `Env` (`picture_frame` and `fade_frame` alike).

use crate::errors::Refused;

/// Whether the frame was made (the original's carry clear): the surface
/// locked (else no frame), the picture copied onto it, unlocked, shown.
pub fn picture_frame(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    if crate::platform::lock_surface(gfx, system) < 1 {
        return Ok(false);
    }
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface(gfx, system),
        )
    }?;
    crate::platform::unlock_surface(gfx);
    Ok(crate::platform::show_frame_now())
}
