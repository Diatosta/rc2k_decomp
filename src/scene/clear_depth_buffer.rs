//! `scene::clear_depth_buffer` (0x44E967): clears the depth buffer over the whole
//! screen.
//!
//! Only with `dword_5CDA8C` not 0: the rectangle `dword_624138` .. = 0, 0, the screen's width
//! and height, and `d3d::clear_viewport` (0x44E9D9: the viewport's Clear with that one
//! rectangle and flag 2, the depth buffer; Direct3D, not ported). Three callers.

// ---- game data ----

// ---- Rust version ----

/// The depth buffer cleared over the whole screen (`d3d::clear_viewport`), unless there
/// is no 3D. What the original leaves (eax 0, ebx the height) its three callers overwrite
/// before reading (each next calls `scene::assign_kind_textures`, which sets both).
pub fn clear_depth_buffer(scene: &crate::scene::owner::Scene) {
    let system = crate::system::owner::system();
    let session = &system.session;
    if scene.objects.tables.has_3d.get() == 0 {
        return;
    }
    let (width, height): (u32, u32) = (session.screen.width.get(), session.screen.height.get());
    let own = &crate::scene::polygon_work::front_end().cleared_rectangle;
    crate::session::write_bits(own, [0, 0, width, height]);
    crate::platform_3d::clear_viewport(own);
}
