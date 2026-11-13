//! One frame of the service area (the callback `screens::run_frame` is given): the picture copied
//! to the back surface, the boxes drawn in a scene (`screens::draw_service_boxes`), then the text
//! and icons (`screens::draw_service_area`) and the frame shown. A failing lock, scene begin or
//! show is reported by the carry of the flags returned. Callers: the frame callbacks of
//! `service::apply_repairs` and its kind.

use crate::errors::Refused;
use crate::framework::CarryZeroFlags;
use crate::owners::Owners;
use crate::platform::{lock_surface, show_frame_now, unlock_surface};

/// The flags it leaves: the lock's carry; on a frame not shown (quitting) the zero flag is
/// clear, on the way through it is set.
pub fn service_area_frame(o: Owners<'static>) -> Result<CarryZeroFlags, Refused> {
    use crate::screens::{draw_service_area, draw_service_boxes};
    const FAILED: CarryZeroFlags = CarryZeroFlags {
        carry: true,
        zero: false,
    };
    if lock_surface(o.gfx, o.system) < 1 {
        return Ok(FAILED);
    }
    crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
        &mut crate::gfx::copy_picture_to_surface::GameSurface(o.gfx, o.system),
    )?;
    unlock_surface(o.gfx);
    crate::scene::begin_scene::begin_scene(o.race, o.d3d, crate::workers::game_scratch());
    draw_service_boxes::draw_service_boxes(o)?;
    crate::scene::end_scene::end_scene(o.d3d);
    if lock_surface(o.gfx, o.system) < 1 {
        return Ok(FAILED);
    }
    draw_service_area::draw_service_area(
        o,
        o.gfx,
        o.race,
        &mut crate::text::draw_text::GameCanvas::new(o.gfx),
    )?;
    unlock_surface(o.gfx);
    if !show_frame_now(o) {
        return Ok(FAILED);
    }
    Ok(CarryZeroFlags {
        carry: false,
        zero: true,
    })
}
