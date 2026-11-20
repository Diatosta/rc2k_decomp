//! One frame of the service area (the callback `screens::run_frame` is given): the picture copied
//! to the back surface, the boxes drawn in a scene (`screens::draw_service_boxes`), then the text
//! and icons (`screens::draw_service_area`) and the frame shown. A failing lock or show ends the
//! frame where it is.

use crate::errors::Refused;
use crate::gfx::with_locked_surface::with_locked_surface;
use crate::owners::Owners;
use crate::shell::frame::show_frame_now;

pub fn service_area_frame(o: Owners<'static>) -> Result<(), Refused> {
    use crate::screens::{draw_service_area, draw_service_boxes};
    let copied = with_locked_surface(
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
        return Ok(());
    }
    crate::scene::begin_scene::begin_scene(o.race, &o.gfx.d3d, o.scene.scratch.game_scratch());
    draw_service_boxes::draw_service_boxes(o)?;
    crate::scene::end_scene::end_scene(&o.gfx.d3d);
    let drawn = with_locked_surface(
        crate::gfx::with_locked_surface::lock(o.gfx, o.system),
        || {
            draw_service_area::draw_service_area(
                o,
                o.gfx,
                o.race,
                &mut crate::text::draw_text::GameCanvas::new(o.gfx),
            )
        },
    )?;
    if drawn.is_none() {
        return Ok(());
    }
    show_frame_now(o);
    Ok(())
}
