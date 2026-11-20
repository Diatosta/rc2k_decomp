//! `menu::picture_frame`: a frame of a still picture: the surface locked, the picture copied to it,
//! unlocked, the frame shown.
//!
//! The frame callback of `menu::championship_welcome` and `menu::a8_welcome` (handed to
//! `screens::run_frame`). The steps are the game's (the surface, the 3D scene and the showing are
//! the platform's), through the owners (`picture_frame` and `fade_frame` alike).

use crate::errors::Refused;

/// Whether the frame was made: the surface locked (else no frame), the picture copied onto it,
/// unlocked, shown.
pub fn picture_frame(o: crate::owners::Owners<'static>) -> Result<bool, Refused> {
    let (gfx, system) = (o.gfx, o.system);
    let copied = crate::gfx::with_locked_surface::with_locked_surface(
        crate::gfx::with_locked_surface::lock(gfx, system),
        || {
            Ok(
                crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
                    &mut crate::gfx::copy_picture_to_surface::GameSurface(gfx, system),
                )?,
            )
        },
    )?;
    if copied.is_none() {
        return Ok(false);
    }
    Ok(crate::shell::frame::show_frame_now(o))
}
