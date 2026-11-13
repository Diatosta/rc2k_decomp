//! Makes a palette that lies between two others.
//! For each entry both colours are taken apart with `pixel_to_components`; each of the three low
//! components becomes first + ((second - first) * the amount >> 8), as a byte; and the result goes
//! back through `components_to_pixel`. The fourth component is not computed: it is whatever the top
//! byte of the result's entry held before, which only matters to a format with a fourth mask.
//! Refuses a count of 0 and null pointers.

use crate::errors::BadInput;
use crate::gfx::formats::Formats;
use crate::gfx::pixel_format::PixelFormat;

// The six arguments, plus the font stores and formats it reads.
#[allow(clippy::too_many_arguments)]
pub fn blend_palette(
    fonts: &crate::gfx::owner::FontStores,
    formats: &Formats,
    first: u32,
    second: u32,
    out: u32,
    count: u32,
    towards_second: u32,
    format: &PixelFormat,
) -> Result<(), BadInput> {
    if count == 0 {
        return Err(BadInput(
            "no entries: the original goes through all 32 bits",
        ));
    }
    if first == 0 || second == 0 || out == 0 {
        return Err(BadInput("a null palette"));
    }
    // The palettes are places of `gfx::glyph_store`.
    let entry_of = |palette: u32, n: u32| -> Result<u32, BadInput> {
        let b = crate::gfx::glyph_store::read_bytes(fonts, palette, 4 * n, 4, "a palette")?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    for n in 0..count {
        let (pixel, scratch) = rc2k_core::gfx::pixels::blend_entry(
            format,
            entry_of(first, n)?,
            entry_of(second, n)?,
            entry_of(out, n)?,
            towards_second,
        );
        if let Some(scratch) = scratch {
            formats.scratch.write(&scratch);
        }
        crate::gfx::glyph_store::write_bytes(fonts, out, 4 * n, &pixel.to_le_bytes(), "a palette")?;
    }
    Ok(())
}
