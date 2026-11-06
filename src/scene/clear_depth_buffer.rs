//! `scene::clear_depth_buffer` (0x44E967): clears the depth buffer over the whole
//! screen.
//!
//! In:  nothing
//! Out: nothing changed without 3D; else eax, ecx, edx as Direct3D leaves them and
//!      ebx = the screen's height
//!
//! Only with `dword_5CDA8C` not 0: the rectangle `dword_624138` .. = 0, 0, the screen's width
//! and height, and `d3d::clear_viewport` (0x44E9D9: the viewport's Clear with that one
//! rectangle and flag 2, the depth buffer; Direct3D, not ported). Three callers.

use crate::scene::owner::scene;
use crate::system::owner::system;

// ---- game data ----

// ---- Rust version ----

/// The depth buffer cleared over the whole screen (`d3d::clear_viewport`), unless there
/// is no 3D. What the original leaves (eax 0, ebx the height) its three callers overwrite
/// before reading (each next calls `scene::assign_kind_textures`, which sets both).
pub fn clear_depth_buffer() {
    let session = &system().session;
    let scene = scene();
    if scene.objects.tables.has_3d.get() == 0 {
        return;
    }
    let (width, height): (u32, u32) = (session.screen.width.get(), session.screen.height.get());
    crate::session::write_bits(
        &scene.drawing.scene.cleared_rectangle,
        [0, 0, width, height],
    );
    crate::platform_3d::clear_viewport();
}
