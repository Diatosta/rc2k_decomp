//! `menu::fade_frame` (0x4A1E60): a frame of the inspection's fades: the picture copied to the locked surface, then in a 3D scene the fade's see-through quad (`screens::draw_fade_overlay`), the frame shown.
//!
//! The frame callback of `menu::fade_in` and 0x4A1DA5 (handed to `screens::run_frame`). The steps
//! are the game's (the surface, the 3D scene and the showing are the platform's), through an `Env`
//! (`picture_frame` and `fade_frame` alike).

use crate::errors::Refused;
use crate::owners::Owners;

/// Whether the frame was made (the original's carry clear): as
/// `picture_frame`, with the fade's overlay drawn in a scene between the unlock and the
/// showing (`scene::begin_scene` never sets the carry).
pub fn fade_frame(o: Owners<'static>) -> Result<bool, Refused> {
    if crate::platform::lock_surface() < 1 {
        return Ok(false);
    }
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface(o.gfx),
        )
    }?;
    crate::platform::unlock_surface();
    crate::scene::begin_scene::begin_scene(o.race);
    crate::screens::draw_fade_overlay::draw_fade_overlay(o)?;
    crate::scene::end_scene::end_scene();
    Ok(crate::platform::show_frame_now())
}
