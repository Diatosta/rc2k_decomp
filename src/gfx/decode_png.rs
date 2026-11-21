//! Decodes a PNG file held in memory, as the game's own decoder does: the file, where the pixels go
//! and the info record to fill (`gfx::png`).
//! The chunks are walked to IEND, trusting every length. IHDR gives the width, the height and the
//! colour type; an IDAT chunk is inflated (with the `miniz_oxide` crate) straight into the pixel
//! area, `(bytes a pixel * width + 1) * height` bytes at most, and its rows are then unfiltered in
//! place and closed up, as three bytes a pixel for colour type 2 and as one byte a pixel for every
//! other type; a PLTE chunk is remembered. At IEND the info record gets whether there was a
//! palette, the width and the height, and with a palette its 0x300 bytes from the PLTE chunk;
//! without one the palette stays as it was. Check sums are not looked at, nor is what the inflating
//! returns. Every PNG file of the game is of colour type 2 with 8 bits and one IDAT chunk;
//! `gfx::read_png` is the one caller.
//! Refused: a file whose length is not known (`read_png` keeps it in `length`); a chunk that runs
//! past the end; more than one IDAT chunk, or one before IHDR; a bit depth other than 8; colour
//! types 4 and 6; a width under 2 or a height of 0; a row filter above 4; data that does not
//! inflate cleanly; a picture too big for the picture buffer.

use crate::errors::BadInput;
use crate::gfx::owner::Png;
use crate::gfx::picture::PICTURE_DWORDS;
use crate::gfx::png::PngInfo;

// The variables for the header, the pixel area and the info block.

/// Returns a status code (1 without a pixel area, 2 without chunks, 0 or 3 otherwise: 3 when the
/// info block lies inside the pixel area, here a record of its own: 0). `pixels` names a picture
/// buffer (`gfx::picture`, 0 none); `info` is the record the decoder fills (`gfx::png::PngInfo`).
pub(super) fn decode_png(
    png: &Png,
    buffers: &crate::gfx::picture::PictureBuffers,
    file: &[u8],
    pixels: u32,
    info: &mut PngInfo,
) -> Result<u32, BadInput> {
    use rc2k_core::gfx::png;
    if pixels == 0 {
        return Ok(1);
    }
    // First the whole walk and the checks without writing anything: what is refused here leaves no
    // trace. (Data that does not inflate, and a row filter above 4, are found out later, with the
    // pixel area written to; the file is untouched.)
    let Some(found) = png::chunks(file)? else {
        return Ok(2);
    };
    // Without a header in the file the variables keep what the last file left in them.
    let header = found.header.unwrap_or_else(|| png::Header {
        width: png.header.width.get(),
        height: png.header.height.get(),
        colour_type: png.header.colour_type.get(),
        compression: 0,
    });
    let area = match found.data {
        Some(_) => png::area_for(&header, PICTURE_DWORDS * 4)?,
        None => 0,
    };
    if let Some(h) = found.header {
        png.header.width.set(h.width);
        png.header.height.set(h.height);
        png.header.compression.set(h.compression);
        png.header.colour_type.set(h.colour_type);
    }
    if let Some((start, length)) = found.data {
        let (row_bytes, step) = png::row_layout(&header);
        // The area as it is now, worked on as a copy and put back.
        let mut buffer = crate::gfx::picture::read_bytes(buffers, pixels, 0, area)?;
        png::inflate(&file[start..start + length], &mut buffer)?;
        // Fixed ("A PNG picture's first row read against stray memory", `wiki/Fixed-bugs.md`):
        // the row above the first is zeros, as PNG has it (the game read the memory before the
        // picture buffer there, which a first row filtered Up, Average or Paeth adds in).
        png::unfilter(&mut buffer, row_bytes, header.height, step, &|_| 0)?;
        crate::gfx::picture::write_bytes(buffers, pixels, 0, &buffer)?;
    }
    info.has_palette = false;
    info.width = header.width;
    info.height = header.height;
    if let Some(start) = found.palette {
        info.has_palette = true;
        let palette = file
            .get(start..start + 0x300)
            .ok_or(BadInput("a palette chunk that runs past the file"))?;
        info.palette.copy_from_slice(palette);
    }
    Ok(0)
}
