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

use crate::d3d::texture_lock::get;
use crate::gfx::screen_state::DRAW_SURFACE;

/// The Rust version (see the header).
pub fn texture_surface_info(record: u32) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::d3d::textures::surface_info`.
    let description = &crate::gfx::upload::UPLOAD.description;
    let kind = get(record + 0xC);
    let f = rc2k_core::d3d::textures::surface_info(
        get(record + 4),
        get(record + 8),
        get(kind + 4),
        crate::gfx::pixel_format::texture_description_format(kind),
        description.pitch.get(),
        description.pixels.get(),
    );
    let surface = &DRAW_SURFACE;
    for (field, v) in [
        (&surface.width, f.width),
        (&surface.height, f.height),
        (&surface.bits_per_pixel, f.bits),
        (&surface.bytes_per_pixel, f.bytes),
        (&surface.row_bytes, f.row_bytes),
        (&surface.row_dwords, f.row_dwords),
        (&surface.format, f.format),
        (&surface.pitch, f.pitch),
        (&surface.pixels, f.pixels),
        (&surface.row_gap, f.gap),
    ] {
        field.set(v);
    }
    crate::gfx::surface::on_address();
    // The clip rectangle is the surface's first four dwords (x, y, width, height).
    crate::gfx::set_clip::set_clip(
        [&surface.x, &surface.y, &surface.width, &surface.height].map(crate::session::Dword::get),
    );
    Ok(())
}
