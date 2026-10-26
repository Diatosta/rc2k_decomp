//! `d3d::texture_surface_info` (0x41AA93): a locked system texture made the surface the
//! blitters draw on (`g_surface`): width and height from the record (esi), bytes and
//! bits a pixel from its pixel kind's description (+0xC), the row's bytes and dwords,
//! the pixels and pitch from the lock's description (`dword_5ABA18`: +0x24, +0x10), the
//! row's gap, the format (the kind's description + 8); then the clip (`gfx::set_clip`
//! on `g_surface`).
//!
//! In:  esi the record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `d3d::lock_texture`.

use crate::d3d::texture_lock::{get, put};
use crate::gfx::screen_state::DRAW_SURFACE;

/// The Rust version (see the header).
pub fn texture_surface_info(record: u32) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::d3d::textures::surface_info`.
    let s = DRAW_SURFACE.x.addr();
    let d = crate::gfx::upload::UPLOAD.description.addr();
    let kind = get(record + 0xC);
    let f = rc2k_core::d3d::textures::surface_info(
        get(record + 4),
        get(record + 8),
        get(kind + 4),
        kind.wrapping_add(8),
        get(d + 0x10),
        get(d + 0x24),
    );
    for (at, v) in [
        (8, f.width),
        (0xC, f.height),
        (0x10, f.bits),
        (0x14, f.bytes),
        (0x18, f.row_bytes),
        (0x1C, f.row_dwords),
        (0x20, f.format),
        (0x24, f.pitch),
        (0x28, f.pixels),
        (0x2C, f.gap),
    ] {
        put(s + at, v);
    }
    unsafe { crate::gfx::set_clip::set_clip(crate::pointers::host(s) as *const [u32; 4]) }
}
