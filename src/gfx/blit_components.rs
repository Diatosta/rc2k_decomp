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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels};
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, TEST_FORMATS};
use crate::gfx::surface::{BASE_RVA, BYTES_PER_PIXEL_RVA, PITCH_RVA};
use crate::gfx::texture_pixels::test_pixels;
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at};
use std::cell::RefCell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::blit_components", rva(0x4065EE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { blit_components(r, &game_target()) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = rows(r, &game_target()).unwrap_or_default();
    regions.push(at(SCRATCH_RVA));
    dispatch_pixels(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

const START: usize = 0x4065EE;
const LEN: usize = 0x87;
const ENTRY_BYTES: [u8; 10] = [0x60, 0x8B, 0xE8, 0x0F, 0xAF, 0x0D, 0xAC, 0x49, 0x4F, 0x00];
const CALL_SITES: [usize; 2] = [0x406632, 0x406658];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = CALL_SITES.iter().map(|site| site - START).collect();
    let copy =
        unsafe { RedirectedCopy::new(game_addr(rva(START)), LEN, &ENTRY_BYTES, &calls, &|_| None) };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The test's surface variables: bytes a pixel, format, pitch, pixels.
    let variables = std::cell::Cell::new([0u32; 4]);
    // The copy's references to the surface's variables made references to the test's.
    unsafe {
        let code = std::slice::from_raw_parts_mut(copy.addr() as *mut u8, LEN);
        let found = [BYTES_PER_PIXEL_RVA, FORMAT_RVA, PITCH_RVA, BASE_RVA]
            .iter()
            .enumerate()
            .map(|(n, &rva)| {
                let from = (game_addr(rva) as u32).to_le_bytes();
                let to = (variables.as_ptr() as u32 + n as u32 * 4).to_le_bytes();
                let mut count = 0;
                for i in 0..LEN - 3 {
                    if code[i..i + 4] == from {
                        code[i..i + 4].copy_from_slice(&to);
                        count += 1;
                    }
                }
                count
            })
            .collect::<Vec<_>>();
        if found != [2, 1, 3, 1] {
            return t.skip("the copy's references to the surface are not where they were read");
        }
    }
    let mut seed = 0x4065_EE00u32;
    for (case, format) in TEST_FORMATS.iter().enumerate() {
        let bytes = if format.masks.iter().all(|&m| m <= 0xFFFF) {
            2u32
        } else {
            4
        };
        for (piece, at_x, at_y) in [([3u32, 2, 5, 4], 1u32, 2u32), ([0, 0, 1, 1], 0, 0)] {
            let (picture_width, picture_height) = (12u32, 9u32);
            let picture = test_pixels(&mut seed, (picture_width * picture_height) as usize);
            let pitch = 13 * bytes + 3;
            let surface = RefCell::new(vec![0xAAu8; (pitch * 10) as usize]);
            let at = surface.borrow_mut().as_mut_ptr() as u32;
            variables.set([bytes, format as *const PixelFormat as u32, pitch, at]);
            let target = Target {
                pixels: at,
                pitch,
                bytes_per_pixel: bytes,
                format: format as *const PixelFormat as u32,
            };
            let before = Regs {
                eax: picture_width * 4,
                ebx: piece.as_ptr() as u32,
                ecx: at_x,
                edx: at_y,
                esi: picture.as_ptr() as u32,
                ..Regs::SENTINEL
            };
            check_pixels(
                &mut t,
                &format!(
                    "case {case}: {format:x?}, {bytes} bytes, piece {piece:?} at {at_x}, {at_y}"
                ),
                copy.addr(),
                before,
                &|| surface.borrow_mut().fill(0xAA),
                &[
                    (at, (pitch * 10) as usize),
                    crate::service::harness::at(SCRATCH_RVA),
                ],
                &|r| {
                    unsafe { blit_components(r, &target) }?;
                    Ok(Left::nothing())
                },
            );
        }
    }
    t.finish()
}
