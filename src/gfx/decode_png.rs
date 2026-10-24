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
//! area, not zeros as the PNG standard has it; the Rust version reads the same bytes.
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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::gfx::block::{game_read, game_write};
use crate::gfx::picture::PICTURE_DWORDS;
use crate::gfx::png::PngInfo;
use crate::gfx::png_file::Slot as PngSlot;
use crate::gfx::png_header::Slot as PngHdrSlot;

// ---- game data ----

const WIDTH_RVA: PngHdrSlot = PngHdrSlot::at(0xA8C190); // dword_A8C190
const HEIGHT_RVA: PngHdrSlot = PngHdrSlot::at(0xA8C194); // dword_A8C194
const COMPRESSION_RVA: PngHdrSlot = PngHdrSlot::at(0xA8C19A); // byte_A8C19A: the header's byte after the colour type
const COLOUR_TYPE_RVA: PngHdrSlot = PngHdrSlot::at(0xA8C1A0); // dword_A8C1A0
/// dword_5BEC30, dword_5BEC34: the file `read_png` holds in memory, and its length.
pub const FILE_RVA: PngSlot = PngSlot::at(0x5BEC30);
pub const FILE_LEN_RVA: PngSlot = PngSlot::at(0x5BEC34);

// ---- Rust version ----
// Converted: `rc2k_core::gfx::png`. Here the game's variables for the header, the pixel area
// (with the row before it, which the first row's filters read) and the info block.

/// Returns what the original returns in eax.
pub unsafe fn decode_png(
    file: &[u8],
    pixels: *mut u8,
    info: *mut PngInfo,
) -> Result<u32, BadInput> {
    use rc2k_core::gfx::png;
    if pixels.is_null() || info.is_null() {
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
    let header = found.header.unwrap_or_else(|| unsafe {
        png::Header {
            width: game_read(WIDTH_RVA),
            height: game_read(HEIGHT_RVA),
            colour_type: game_read(COLOUR_TYPE_RVA),
            compression: 0,
        }
    });
    let pixel_bytes = png::PIXEL_BYTES[header.colour_type as usize & 7];
    let area = match found.data {
        Some(_) => png::area_for(&header, PICTURE_DWORDS * 4)?,
        None => 0,
    };
    unsafe {
        if let Some(h) = found.header {
            game_write(WIDTH_RVA, h.width);
            game_write(HEIGHT_RVA, h.height);
            game_write(COMPRESSION_RVA, h.compression);
            game_write(COLOUR_TYPE_RVA, h.colour_type);
        }
        if let Some((start, length)) = found.data {
            let (row_bytes, step) = png::row_layout(&header);
            let buffer = std::slice::from_raw_parts_mut(pixels, area);
            png::inflate(&file[start..start + length], buffer)?;
            // The row above the first is the memory before the area, read only where the
            // original reads it.
            let before = pixels.wrapping_sub(row_bytes) as *const u8;
            png::unfilter(buffer, row_bytes, header.height, step, &|i| {
                before.wrapping_add(i).read()
            })?;
        }
        (&raw mut (*info).has_palette).write(0);
        (&raw mut (*info).width).write_unaligned(header.width);
        (&raw mut (*info).height).write_unaligned(header.height);
        if let Some(start) = found.palette {
            (&raw mut (*info).has_palette).write(1);
            std::ptr::copy(
                file[start..].as_ptr(),
                (&raw mut (*info).palette).cast(),
                0x300,
            );
        }
    }
    let (info, pixels) = (info as u32, pixels as u32);
    let end = pixel_bytes
        .wrapping_mul(header.width)
        .wrapping_mul(header.height)
        .wrapping_add(pixels);
    Ok(if info >= pixels && info < end { 3 } else { 0 })
}
