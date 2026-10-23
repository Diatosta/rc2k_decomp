//! `gfx::rescale_to_surface` (0x4063B7): draws the decoded picture onto the surface at the
//! surface's size.
//!
//! In:  nothing
//! Out: every register preserved
//! `gfx::rescale_picture`'s arithmetic with other ends: the rows are those of the decoded
//! picture where it lies (`g_decodedPixels`, `g_decodedStride` apart, listed with
//! `gfx::fill_row_table`), and every pixel made goes to the surface in the surface's
//! format: its three parts are put in `components` (0x4F50BC, whose fourth byte is left
//! as it was), `gfx::components_to_pixel` with the surface's format makes the pixel, and
//! it is stored as a word on a 16-bit surface, else as a dword; the surface's gap is
//! added after each row.
//! Three call sites, all in `gfx::show_picture_file`, with the surface locked.
//! Refuses a decoded picture or a surface without width or height, more rows than the row
//! table holds, and missing pixels or format.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::rescale_picture::STEP_X_RVA;

// ---- game data ----

const DECODED_WIDTH_RVA: usize = rva(0x519124); // g_decodedWidth
const DECODED_HEIGHT_RVA: usize = rva(0x519128); // g_decodedHeight
const DECODED_STRIDE_RVA: usize = rva(0x51913C); // g_decodedStride
const DECODED_PIXELS_RVA: usize = rva(0x519140); // g_decodedPixels
const COLUMNS_LEFT: usize = 0x18;
const ROWS_LEFT: usize = 0x1C;
const COMPONENTS: usize = 0x20;
/// `g_surface` (0x4F4998), twelve dwords, and the fields used here.
pub const SURFACE_RVA: usize = rva(0x4F4998);
const WIDTH: usize = 0x08;
const HEIGHT: usize = 0x0C;
const BITS: usize = 0x10;
const FORMAT: usize = 0x20;
const PIXELS: usize = 0x28;
const ROW_GAP: usize = 0x2C;

// ---- Rust version ----

/// `surface` is the twelve dwords of a surface's description.
pub unsafe fn rescale_to_surface(surface: *const u32) -> Result<(), Refused> {
    unsafe {
        let field = |offset: usize| surface.byte_add(offset).read_unaligned();
        let (from_width, from_height): (u32, u32) =
            (game_read(DECODED_WIDTH_RVA), game_read(DECODED_HEIGHT_RVA));
        let (width, height) = (field(WIDTH), field(HEIGHT));
        let from: u32 = game_read(DECODED_PIXELS_RVA);
        if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
            return Err(BadInput("a picture or a surface without width or height").into());
        }
        if from_height > ROW_TABLE_ROOM || from == 0 || field(PIXELS) == 0 || field(FORMAT) == 0 {
            return Err(BadInput("a decoded picture or a surface the rescaler cannot take").into());
        }
        // The count was checked above.
        let _ = fill_row_table(from, game_read(DECODED_STRIDE_RVA), from_height);
        // Converted: `rc2k_core::gfx::picture::rescale`; each pixel's parts through the
        // components variable, the last left there with its fourth byte as it was.
        let format = (field(FORMAT) as *const PixelFormat).read_unaligned();
        let words = field(BITS) == 0x10;
        let mut out = field(PIXELS) as *mut u8;
        let (width_pixels, gap) = (width, field(ROW_GAP) as usize);
        let mut column = 0u32;
        let work = rc2k_core::gfx::picture::rescale(
            &rc2k_core::gfx::picture::Sizes {
                from_width,
                from_height,
                width,
                height,
            },
            &|row, byte| (row_address(row).wrapping_add(byte) as *const u8).read(),
            &mut |parts| {
                for (part, &byte) in parts.iter().enumerate() {
                    game_write(STEP_X_RVA + COMPONENTS + part, byte);
                }
                let pixel = components_to_pixel(&format, game_read(STEP_X_RVA + COMPONENTS));
                if words {
                    (out as *mut u16).write_unaligned(pixel as u16);
                    out = out.add(2);
                } else {
                    (out as *mut u32).write_unaligned(pixel);
                    out = out.add(4);
                }
                column += 1;
                if column == width_pixels {
                    column = 0;
                    out = out.wrapping_add(gap);
                }
            },
        )
        .expect("the sizes were checked");
        game_write(
            STEP_X_RVA,
            [
                work.step_x,
                work.step_x_fraction,
                work.step_y,
                work.step_y_fraction,
                work.weight_x,
                work.weight_y,
            ],
        );
        game_write(STEP_X_RVA + COLUMNS_LEFT, 0u32);
        game_write(STEP_X_RVA + ROWS_LEFT, 0u32);
        Ok(())
    }
}
