//! `gfx::decode_png` (0x4D3040): decodes a PNG file held in memory.
//!
//! In (on the stack, a C function): the file, where the pixels go, the `PngInfo` to fill
//! Out: eax = 0, or 3 if the info block lies inside the pixels, 1 if an argument is null, 2
//!      if the file does not begin as a PNG; ecx, edx whatever; the rest preserved
//! The original walks the chunks to IEND, trusting every length. IHDR gives the width, the
//! height and the colour type; an IDAT chunk is inflated (zlib 1.1.3) straight into the
//! pixel area, `(bytes a pixel * width + 1) * height` bytes at most, and its rows are then
//! unfiltered in place and closed up, as three bytes a pixel for colour type 2 and as one
//! byte a pixel for every other type; a PLTE chunk is remembered. At IEND the info block
//! gets 0 or 1 in its first byte, the width and the height, and with a palette 0x300 bytes
//! from the PLTE chunk. Check sums are not looked at, nor is what the inflating returns.
//! For the first row the "row above" of filters 2, 3 and 4 is the memory before the pixel
//! area, not zeros as the PNG standard has it; fixed in phase 12 (2026-10-27): zeros
//! (`notes/bugs.md`, `wiki/Fixed-bugs.md`).
//! The Rust version inflates with the `miniz_oxide` crate and does the rest as the original
//! does. What it does not do: use the info block and what follows it as zlib's work area
//! (the original's zlib takes its memory from there), so that memory is left alone; and
//! keep zlib's own variables (0xA8C1A4 on).
//! One caller, `gfx::read_png`. Every PNG file of the game is of colour type 2 with 8 bits
//! and one IDAT chunk.
//! Refuses, and leaves to the original: a file whose length is not known (the function is
//! not told it; `read_png` keeps it in `dword_5BEC34`); a chunk that runs past the end;
//! more than one IDAT chunk, or one before IHDR; a bit depth other than 8; colour types 4
//! and 6 (the original takes them for one byte a pixel); a width under 2 or a height of 0
//! (the original counts through all 32 bits); a row filter above 4; data that does not
//! inflate cleanly; a picture too big for the picture buffer. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::picture::PICTURE_DWORDS;
use crate::gfx::png_header::HEADER;
use crate::pointers::GamePtr;

// ---- game data ----

// ---- Rust version ----
// Converted: `rc2k_core::gfx::png`. Here the game's variables for the header, the pixel area
// and the info block.

/// Returns what the original returns in eax. `pixels` names a picture buffer
/// (`gfx::picture`, 0 none).
pub fn decode_png(file: &[u8], pixels: u32, info_at: GamePtr) -> Result<u32, BadInput> {
    use rc2k_core::gfx::png;
    if pixels == 0 || info_at.is_null() {
        return Ok(1);
    }
    // First the whole walk and the checks without writing anything: what is refused here
    // leaves no trace. (Data that does not inflate, and a row filter above 4, are found
    // out later, with the pixel area written to; the file is untouched, and the original
    // starts from it.)
    let Some(found) = png::chunks(file)? else {
        return Ok(2);
    };
    // Without a header in the file the variables keep what the last file left in them.
    let header = found.header.unwrap_or_else(|| png::Header {
        width: HEADER.width.get(),
        height: HEADER.height.get(),
        colour_type: HEADER.colour_type.get(),
        compression: 0,
    });
    let pixel_bytes = png::PIXEL_BYTES[header.colour_type as usize & 7];
    let area = match found.data {
        Some(_) => png::area_for(&header, PICTURE_DWORDS * 4)?,
        None => 0,
    };
    if let Some(h) = found.header {
        HEADER.width.set(h.width);
        HEADER.height.set(h.height);
        HEADER.compression.set(h.compression);
        HEADER.colour_type.set(h.colour_type);
    }
    if let Some((start, length)) = found.data {
        let (row_bytes, step) = png::row_layout(&header);
        // The area as it is now, worked on as a copy and put back.
        let mut buffer = crate::gfx::picture::read_bytes(pixels, 0, area)?;
        png::inflate(&file[start..start + length], &mut buffer)?;
        // The row above the first is zeros, as PNG has it. Our fix (phase 12): the original
        // reads the memory before the picture buffer there (its heap block's header and the
        // page before), which a first row filtered Up, Average or Paeth adds in.
        png::unfilter(&mut buffer, row_bytes, header.height, step, &|_| 0)?;
        crate::gfx::picture::write_bytes(pixels, 0, &buffer)?;
    }
    let what = "the PNG info block";
    info_at.write(0u8, what)?;
    info_at.offset(4).write(header.width, what)?;
    info_at.offset(8).write(header.height, what)?;
    if let Some(start) = found.palette {
        info_at.write(1u8, what)?;
        let palette = file
            .get(start..start + 0x300)
            .ok_or(BadInput("a palette chunk that runs past the file"))?;
        info_at.offset(12).write_bytes(palette, what)?;
    }
    let end = pixel_bytes
        .wrapping_mul(header.width)
        .wrapping_mul(header.height)
        .wrapping_add(pixels);
    // The info block's place as a number only to compare it with the pixels' span.
    let info = info_at.addr();
    Ok(if info >= pixels && info < end { 3 } else { 0 })
}
