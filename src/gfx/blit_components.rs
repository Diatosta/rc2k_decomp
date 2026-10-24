//! `gfx::blit_components` (0x4065EE): a piece of a picture of components onto the
//! surface, in the surface's pixel format.
//!
//! In:  eax = the picture's bytes from one row to the next, ebx = the piece (x, y,
//!      width, height in the picture), ecx, edx = where on the surface, esi = the
//!      picture's pixels
//! Out: every register preserved
//!
//! The surface's place = its pixels (`g_surface.pixels`) + y times its pitch + x times
//! its bytes a pixel; the picture's = esi + the piece's y times eax + its x times 4.
//! Each row: the piece's width of pixels through `gfx::components_to_pixel` with the
//! surface's format (`g_surface.format`), stored as words on a surface of two bytes a
//! pixel, else as dwords; then both move on a row. No clipping.
//! One caller, `scene::upload_plate`, with the texture made the surface.
//! Refuses a piece of no width or height (the original counts through all 32 bits),
//! and no surface, format, piece or picture.
//! Compared in shadow mode (`dispatch_pixels`).
//! Self-test: a copy whose four reads of the surface's variables read the test's, nine
//! formats on two sizes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::surface::{BASE_RVA, BYTES_PER_PIXEL_RVA, PITCH_RVA};
use crate::service::harness::Region;

// ---- game data ----

/// `g_surface.format`: the address of the surface's `PixelFormat`.
const FORMAT_RVA: usize = rva(0x4F49B8);

// ---- Rust version ----

/// The surface as the function reads it.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub pixels: u32,
    pub pitch: u32,
    pub bytes_per_pixel: u32,
    pub format: u32,
}

pub fn game_target() -> Target {
    unsafe {
        Target {
            pixels: game_read(BASE_RVA),
            pitch: game_read(PITCH_RVA),
            bytes_per_pixel: game_read(BYTES_PER_PIXEL_RVA),
            format: game_read(FORMAT_RVA),
        }
    }
}

/// What the original gets in its registers: ebx the piece's rectangle (x, y, width,
/// height), ecx, edx where it goes on the surface, esi the picture's pixels, eax the
/// picture's row bytes.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub rectangle: u32,
    pub x: u32,
    pub y: u32,
    pub pixels: u32,
    pub row_bytes: u32,
}

/// The places it writes: one region a row.
fn rows(p: &Piece, to: &Target) -> Option<Vec<Region>> {
    if p.rectangle == 0 || p.pixels == 0 || to.pixels == 0 || to.format == 0 {
        return None;
    }
    let [_, _, width, height] = unsafe { (p.rectangle as *const [u32; 4]).read_unaligned() };
    if width == 0 || height == 0 {
        return None;
    }
    let bytes = if to.bytes_per_pixel == 2 { 2 } else { 4 };
    let first =
        p.y.wrapping_mul(to.pitch)
            .wrapping_add(p.x.wrapping_mul(to.bytes_per_pixel))
            .wrapping_add(to.pixels);
    Some(
        (0..height)
            .map(|row| {
                (
                    first.wrapping_add(row.wrapping_mul(to.pitch)),
                    width as usize * bytes,
                )
            })
            .collect(),
    )
}

pub unsafe fn blit_components(p: &Piece, to: &Target) -> Result<(), Refused> {
    if rows(p, to).is_none() {
        return Err(BadInput("a piece of nothing, or no surface, format, piece or picture").into());
    }
    unsafe {
        let [x, y, width, height] = (p.rectangle as *const [u32; 4]).read_unaligned();
        let format = (to.format as *const PixelFormat).read_unaligned();
        let mut target =
            p.y.wrapping_mul(to.pitch)
                .wrapping_add(p.x.wrapping_mul(to.bytes_per_pixel))
                .wrapping_add(to.pixels);
        let mut source = p
            .pixels
            .wrapping_add(y.wrapping_mul(p.row_bytes))
            .wrapping_add(x.wrapping_mul(4));
        // Converted: `rc2k_core::gfx::pixels::pack_row`, a row at a time.
        let words = to.bytes_per_pixel == 2;
        let size = if words { 2 } else { 4 };
        for _ in 0..height {
            let row = std::slice::from_raw_parts(source as *const u8, width as usize * 4);
            let out = std::slice::from_raw_parts_mut(target as *mut u8, width as usize * size);
            let scratch =
                rc2k_core::gfx::pixels::pack_row(row, out, width as usize, &format, words)?;
            if let Some(scratch) = scratch {
                crate::framework::game_write(crate::gfx::pixel_format::SCRATCH_RVA, scratch);
            }
            target = target.wrapping_add(to.pitch);
            source = source.wrapping_add(p.row_bytes);
        }
    }
    Ok(())
}
