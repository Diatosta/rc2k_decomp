//! `d3d::texture_surface_info`: a locked system texture made the surface the blitters draw on:
//! width and height from the record, bytes and bits a pixel from its pixel kind's description
//! (+0xC), the row's bytes and dwords, the pixels and pitch from the lock's description, the
//! row's gap, the format (the kind's description + 8); then the clip (`gfx::set_clip` on the
//! surface). Takes the record. Caller: `d3d::lock_texture`.

use crate::gfx::owner::Gfx;
use crate::gfx::texture_upload::RecordCell;

pub fn texture_surface_info(record: &RecordCell, gfx: &Gfx) -> Result<(), crate::errors::BadInput> {
    // Logic: `rc2k_core::d3d::textures::surface_info`.
    let description = &gfx.card.upload.description;
    let kind = record.format();
    let f = rc2k_core::d3d::textures::surface_info(
        record.width(),
        record.height(),
        bytes_per_pixel(kind, gfx)?,
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
    crate::gfx::surface::on_address(gfx);
    // The clip rectangle is the surface's first four dwords (x, y, width, height).
    crate::gfx::set_clip::set_clip(
        [&surface.x, &surface.y, &surface.width, &surface.height]
            .map(crate::memory::session::Dword::get),
        gfx,
    );
    Ok(())
}

/// The bytes a pixel of the description a record's kind field names.
fn bytes_per_pixel(kind: u32, gfx: &Gfx) -> Result<u32, crate::errors::BadInput> {
    gfx.card
        .texture_tables
        .description(kind)
        .map(|d| d.get().bytes_per_pixel)
        .ok_or(crate::errors::BadInput(
            "a texture's kind that names no description",
        ))
}
