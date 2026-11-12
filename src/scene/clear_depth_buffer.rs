//! `scene::clear_depth_buffer` (0x44E967): clears the depth buffer over the whole
//! screen.
//!
//! Only with `ObjectTables.d3d_drawing_on` not 0: the rectangle `SceneDraw.cleared_rectangle[0]` .. = 0, 0, the screen's width
//! and height, and `d3d::clear_viewport` (0x44E9D9: the viewport's Clear with that one
//! rectangle and flag 2, the depth buffer; Direct3D, not ported). Three callers.

// ---- game data ----

// ---- Rust version ----

/// The depth buffer cleared over the whole screen (`d3d::clear_viewport`), unless there
/// is no 3D.
pub fn clear_depth_buffer(
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    part: &crate::workers::PartScratch,
    polygons: &crate::scene::polygon_work::PolygonWork,
) {
    let session = &system.session;
    if scene.objects.tables.get().d3d_drawing_on.get() == 0 {
        return;
    }
    let (width, height): (u32, u32) = (session.screen.width.get(), session.screen.height.get());
    let own = &polygons.cleared_rectangle;
    crate::session::write_bits(own, [0, 0, width, height]);
    crate::platform_3d::clear_viewport(part, own);
}
