//! `gfx::convert_picture` (0x406126): turns the picture in the picture buffer, 32-bit pixels
//! of four components, into pixels of the surface's format, in place.
//!
//! In:  nothing
//! Out: every register preserved
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
//! calls the one the table names, and shadow mode then runs the original alone. Refuses a null
//! buffer and a width or height of 0 (the original's loops count through all 32 bits). No
//! limit on width * height, as in the original. The name is a reading.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, call_game, game_addr, game_read, game_write};
use crate::gfx::components_to_4444_rows::{COMPONENTS_TO_4444_ROWS, components_to_4444_rows};
use crate::gfx::copy_pixel_rows::{COPY_PIXEL_ROWS, copy_pixel_rows};
use crate::gfx::install_converters::CONVERTERS_RVA;
use crate::gfx::picture::{
    FLAGS_RVA, HEIGHT_RVA, ROW_BYTES_RVA, SCREEN_BITS_RVA, WIDTH_RVA, picture,
};
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, SURFACE_FORMAT_RVA};
use crate::screens::harness::Left;

// ---- game data ----

const FORMAT_NUMBER: usize = 0x14; // in the surface's format

/// The surface format's number.
fn format_number() -> u32 {
    unsafe { game_read(SURFACE_FORMAT_RVA + FORMAT_NUMBER) }
}

/// The converter the game has for the surface's format, or 0.
fn converter() -> u32 {
    unsafe { game_read(CONVERTERS_RVA + format_number() as usize * 4) }
}

// ---- Rust version ----

/// `screen_bits` is `dword_4F4954`; `converter` the table's entry for the surface's format.
pub unsafe fn convert_picture(screen_bits: u32, converter: u32) -> Result<(), BadInput> {
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
                return Ok(());
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
                return Ok(());
            }
        } else if converter != 0 {
            let mut g = Regs {
                eax: format_number(),
                ebx: game_addr(SURFACE_FORMAT_RVA) as u32,
                ecx: width.wrapping_mul(height),
                edx: 1,
                esi: buffer as u32,
                edi: buffer as u32,
                ebp: 0,
                ..Regs::default()
            };
            call_game(&mut g, converter as usize);
            if screen_bits != 16 {
                return Ok(());
            }
        } else {
            // Converted: `rc2k_core::gfx::pixels::convert_in_place`.
            let format: PixelFormat = game_read(SURFACE_FORMAT_RVA);
            let pixels = width as usize * height as usize;
            let bytes = std::slice::from_raw_parts_mut(buffer, pixels * 4);
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
                return Ok(());
            }
        }
        game_write(FLAGS_RVA, game_read::<u32>(FLAGS_RVA) & 0xFFFF_F0FF | 0x200);
        game_write(ROW_BYTES_RVA, game_read::<u32>(ROW_BYTES_RVA) >> 1);
        Ok(())
    }
}

/// With the screen's bits and the surface format's converter as they are now.
pub unsafe fn convert_picture_now() -> Result<(), BadInput> {
    unsafe { convert_picture(game_read(SCREEN_BITS_RVA), converter()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { convert_picture(game_read(SCREEN_BITS_RVA), converter()) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if picture().is_null() {
        return Err(BadInput("no picture buffer").into());
    }
    crate::gfx::harness::pixels_entry(r, &rust)
}

// ---- self-test: a copy of the original that reads the screen's bits and the converter table
// from test variables, on a small picture in the game's own buffer ----
