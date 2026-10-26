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
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::{BadInput, Refused};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::decoder::DECODER;
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::rescale_picture::RESCALE;

// ---- game data ----

// The fields used here of `g_surface` (`DRAW_SURFACE`, from 0x4F4998), twelve dwords.
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
        let (from_width, from_height): (u32, u32) = (DECODER.width.get(), DECODER.height.get());
        let (width, height) = (field(WIDTH), field(HEIGHT));
        let from: u32 = DECODER.pixels.get();
        if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
            return Err(BadInput("a picture or a surface without width or height").into());
        }
        if from_height > ROW_TABLE_ROOM || from == 0 || field(PIXELS) == 0 || field(FORMAT) == 0 {
            return Err(BadInput("a decoded picture or a surface the rescaler cannot take").into());
        }
        // The count was checked above.
        let _ = fill_row_table(from, DECODER.stride.get(), from_height);
        // Converted: `rc2k_core::gfx::picture::rescale`; each pixel's parts through the
        // components variable, the last left there with its fourth byte as it was.
        let format = (crate::pointers::host(field(FORMAT)) as *const PixelFormat).read_unaligned();
        let words = field(BITS) == 0x10;
        let mut out = crate::pointers::host(field(PIXELS));
        let (width_pixels, gap) = (width, field(ROW_GAP) as usize);
        let mut column = 0u32;
        let window = crate::pointers::Window::new();
        let work = rc2k_core::gfx::picture::rescale(
            &rc2k_core::gfx::picture::Sizes {
                from_width,
                from_height,
                width,
                height,
            },
            &|row, byte| {
                window.read_or_end(row_address(row).wrapping_add(byte), "a picture's pixel")
            },
            &mut |parts| {
                for (part, &byte) in parts.iter().enumerate() {
                    RESCALE.components.set(part, byte);
                }
                let pixel = components_to_pixel(&format, RESCALE.components.get_u32(0));
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
        RESCALE.step_x.set(work.step_x);
        RESCALE.step_x_fraction.set(work.step_x_fraction);
        RESCALE.step_y.set(work.step_y);
        RESCALE.step_y_fraction.set(work.step_y_fraction);
        RESCALE.weight_x.set(work.weight_x);
        RESCALE.weight_y.set(work.weight_y);
        RESCALE.columns_left.set(0);
        RESCALE.rows_left.set(0);
        Ok(())
    }
}
