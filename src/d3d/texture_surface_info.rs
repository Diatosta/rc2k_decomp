//! `d3d::texture_surface_info` (0x41AA93): a locked system texture made the surface the
//! blitters draw on (`g_surface`): width and height from the record (esi), bytes and
//! bits a pixel from its pixel kind's description (+0xC), the row's bytes and dwords,
//! the pixels and pitch from the lock's description (`dword_5ABA18`: +0x24, +0x10), the
//! row's gap, the format (the kind's description + 8); then the clip (`gfx::set_clip`
//! on `g_surface`).
//!
//! Takes: the record
//!
//! Caller: `d3d::lock_texture`.

use crate::d3d::texture_lock::field;
use crate::gfx::owner::gfx;

pub fn texture_surface_info(
    record: crate::pointers::GamePtr,
) -> Result<(), crate::errors::BadInput> {
    let gfx = gfx();
    // Converted: `rc2k_core::d3d::textures::surface_info`.
    let description = &gfx.card.upload.description;
    let kind = field(record, 0xC);
    let f = rc2k_core::d3d::textures::surface_info(
        field(record, 4),
        field(record, 8),
        bytes_per_pixel(kind),
        crate::gfx::pixel_format::texture_description_format(kind, gfx),
        description.pitch.get(),
        description.pixels.get(),
    );
    let surface = &gfx.drawing.draw_surface;
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

/// The bytes a pixel of the description a record's kind field names: a handle, else (a kind
/// read past the table) the dword at +4 of that number as an address.
fn bytes_per_pixel(kind: u32) -> u32 {
    match gfx().card.texture_tables.description(kind) {
        Some(d) => d.get().bytes_per_pixel,
        None => crate::d3d::texture_lock::get(kind.wrapping_add(4)),
    }
}
