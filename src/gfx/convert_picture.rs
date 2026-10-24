//! `gfx::convert_picture` (0x406126): turns the picture in the picture buffer, 32-bit pixels
//! of four components, into pixels of the surface's format, in place.
//!
//! In:  nothing
//! Out: every register preserved; the carry flag as its last instruction leaves it (no
//!      result, but `menu::inspection_area_loop` raises a fatal error on it after a failed
//!      load: `CarryLeft`)
//! The surface's format (`g_surfaceFormat`, 0x4F496C) has a number at +0x14. If the table
//! `dword_4F5288` holds a converter for that number, it is called (ebx = the format, esi =
//! edi = the buffer, eax = the number, ecx = width * height, edx = 1, ebp = 0). Otherwise
//! every pixel goes through `components_to_pixel`: with a 16-bit screen the results are
//! written as words from the buffer's start, with any other as dwords. With a 16-bit screen,
//! whichever way the pixels were made, bits 8 .. 11 of the picture's flags become 2 and its
//! row length is halved.
//! 14 callers. Of the converters (the table is filled by `gfx::install_converters`) two are
//! ported and called as Rust, `copy_pixel_rows` (the owner's 32-bit screen: nothing to do)
//! and `components_to_4444_rows`; the eight MMX and 3DNow ones are not: the Rust version
//! calls the one the table names. Refuses a null
//! buffer and a width or height of 0 (the original's loops count through all 32 bits). No
//! limit on width * height, as in the original. The name is a reading.
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::BadInput;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::components_to_4444_rows::{COMPONENTS_TO_4444_ROWS, components_to_4444_rows};
use crate::gfx::copy_pixel_rows::{COPY_PIXEL_ROWS, copy_pixel_rows};
use crate::gfx::install_converters::CONVERTERS_RVA;
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA, picture};
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, surface_format_addr};
use crate::session::SCREEN;

// ---- game data ----

const FORMAT_NUMBER: usize = 0x14; // in the surface's format

/// The surface format's number.
fn format_number() -> u32 {
    crate::session::SCREEN.format[FORMAT_NUMBER / 4].get()
}

/// The converter the game has for the surface's format, or 0.
fn converter() -> u32 {
    unsafe { game_read(CONVERTERS_RVA + format_number() as usize * 4) }
}

// ---- Rust version ----

/// The carry flag the original leaves (true: set).
pub type CarryLeft = bool;

/// `screen_bits` is `dword_4F4954`; `converter` the table's entry for the surface's format.
pub unsafe fn convert_picture(screen_bits: u32, converter: u32) -> Result<CarryLeft, BadInput> {
    unsafe {
        let buffer = picture();
        let (width, height): (u32, u32) = (game_read(WIDTH_RVA), game_read(HEIGHT_RVA));
        if buffer.is_null() {
            return Err(BadInput("no picture buffer"));
        }
        if converter == 0 && (width == 0 || height == 0) {
            return Err(BadInput("a picture without a width or a height"));
        }
        if converter as usize == game_addr(COPY_PIXEL_ROWS) {
            // The picture is already what the surface wants: from and to the same place,
            // `copy_pixel_rows` does nothing.
            copy_pixel_rows(
                buffer as u32,
                buffer as u32,
                width.wrapping_mul(height),
                1,
                0,
            )?;
            if screen_bits != 16 {
                return Ok(screen_bits < 16);
            }
        } else if converter as usize == game_addr(COMPONENTS_TO_4444_ROWS) {
            if width.wrapping_mul(height) == 0 {
                return Err(BadInput("a picture without a width or a height"));
            }
            components_to_4444_rows(
                buffer as u32,
                buffer as u32,
                width.wrapping_mul(height),
                1,
                0,
            )?;
            if screen_bits != 16 {
                return Ok(screen_bits < 16);
            }
        } else if converter != 0 {
            // The table's converter: an MMX / 3DNow one by its Rust version (the table
            // holds no other: `gfx::install_converters`).
            let at = (converter as usize)
                .wrapping_sub(game_addr(0))
                .wrapping_add(crate::framework::IMAGE_BASE) as u32;
            let Some(c) = crate::gfx::mmx_converters::converter_at(at) else {
                return Err(BadInput("a converter with no Rust version in the table"));
            };
            let buffer = buffer as u32;
            match crate::gfx::mmx_converters::convert_rows(
                buffer,
                buffer,
                width.wrapping_mul(height),
                1,
                0,
                c,
            ) {
                Ok(()) => {}
                Err(crate::errors::Refused::BadInput(why)) => return Err(why),
                Err(crate::errors::Refused::Fatal(_)) => {
                    return Err(BadInput("a converter's fatal error"));
                }
            }
            if screen_bits != 16 {
                return Ok(screen_bits < 16);
            }
        } else {
            // Converted: `rc2k_core::gfx::pixels::convert_in_place`.
            let format: PixelFormat =
                (surface_format_addr() as *const PixelFormat).read_unaligned();
            let pixels = width as usize * height as usize;
            let bytes = std::slice::from_raw_parts_mut(buffer, pixels * 4);
            // `components_to_pixel`'s carry for the last pixel: clear for 0 and for a format
            // of rotations (+0 negative), set after its loop over the masks (`shr ecx, 1`
            // shifting out the last bit).
            let last = u32::from_le_bytes(bytes[(pixels - 1) * 4..pixels * 4].try_into().unwrap());
            let carry = last != 0 && (crate::session::SCREEN.format[0].get() as i32) >= 0;
            let scratch = rc2k_core::gfx::pixels::convert_in_place(
                bytes,
                pixels,
                &format,
                screen_bits == 16,
            )?;
            if let Some(scratch) = scratch {
                game_write(SCRATCH_RVA, scratch);
            }
            if screen_bits != 16 {
                return Ok(carry);
            }
        }
        game_write(FLAGS_RVA, game_read::<u32>(FLAGS_RVA) & 0xFFFF_F0FF | 0x200);
        let row_bytes = game_read::<u32>(ROW_BYTES_RVA);
        game_write(ROW_BYTES_RVA, row_bytes >> 1);
        // `shr`: the bit shifted out.
        Ok(row_bytes & 1 != 0)
    }
}

/// With the screen's bits and the surface format's converter as they are now.
pub unsafe fn convert_picture_now() -> Result<CarryLeft, BadInput> {
    if picture().is_null() {
        return Err(BadInput("no picture buffer"));
    }
    unsafe { convert_picture(SCREEN.bits.get(), converter()) }
}
