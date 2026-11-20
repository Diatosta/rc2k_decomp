//! `menu::fade_frame`: a frame of the inspection's fades: the picture copied to the locked
//! surface, then in a 3D scene the fade's see-through quad (`screens::draw_fade_overlay`), the
//! frame shown.
//!
//! The frame callback of `menu::fade_in` and the fade out (handed to `screens::run_frame`). The
//! steps are the game's (the surface, the 3D scene and the showing are the platform's), through
//! the owners (`picture_frame` and `fade_frame` alike).

use crate::errors::Refused;
use crate::owners::Owners;

/// Whether the frame was made: as `picture_frame`, with the fade's overlay drawn in a scene
/// between the unlock and the showing (`scene::begin_scene` never fails).
pub fn fade_frame(o: Owners<'static>) -> Result<bool, Refused> {
    let copied = crate::gfx::with_locked_surface::with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            Ok(
                crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
                    &mut crate::gfx::copy_picture_to_surface::GameSurface(o.gfx, o.system),
                )?,
            )
        },
    )?;
    if copied.is_none() {
        return Ok(false);
    }
    crate::scene::begin_scene::begin_scene(o.race, &o.gfx.d3d, o.scene.scratch.game_scratch());
    crate::screens::draw_fade_overlay::draw_fade_overlay(o)?;
    crate::scene::end_scene::end_scene(&o.gfx.d3d);
    Ok(crate::shell::frame::show_frame_now(o))
}
