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
use crate::framework::{Regs, game_read, rva};
use crate::gfx::harness::pixels_entry;
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::surface::{BASE_RVA, BYTES_PER_PIXEL_RVA, PITCH_RVA};
use crate::screens::harness::Left;
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

/// The places it writes: one region a row.
fn rows(r: &Regs, to: &Target) -> Option<Vec<Region>> {
    if r.ebx == 0 || r.esi == 0 || to.pixels == 0 || to.format == 0 {
        return None;
    }
    let [_, _, width, height] = unsafe { (r.ebx as *const [u32; 4]).read_unaligned() };
    if width == 0 || height == 0 {
        return None;
    }
    let bytes = if to.bytes_per_pixel == 2 { 2 } else { 4 };
    let first = r
        .edx
        .wrapping_mul(to.pitch)
        .wrapping_add(r.ecx.wrapping_mul(to.bytes_per_pixel))
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

pub unsafe fn blit_components(r: &Regs, to: &Target) -> Result<(), Refused> {
    if rows(r, to).is_none() {
        return Err(BadInput("a piece of nothing, or no surface, format, piece or picture").into());
    }
    unsafe {
        let [x, y, width, height] = (r.ebx as *const [u32; 4]).read_unaligned();
        let format = (to.format as *const PixelFormat).read_unaligned();
        let mut target = r
            .edx
            .wrapping_mul(to.pitch)
            .wrapping_add(r.ecx.wrapping_mul(to.bytes_per_pixel))
            .wrapping_add(to.pixels);
        let mut source = r
            .esi
            .wrapping_add(y.wrapping_mul(r.eax))
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
            source = source.wrapping_add(r.eax);
        }
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { blit_components(r, &game_target()) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}
