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
//! Self-test: a redirected copy whose surface is the test's.
//! Refuses a decoded picture or a surface without width or height, more rows than the
//! row table holds, and missing pixels or format.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, ROW_TABLE_RVA, fill_row_table, row_address};
use crate::gfx::harness::dispatch_pixels;
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, TEST_FORMATS};
use crate::gfx::rescale_picture::{STEP_X_RVA, blend, step};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::rescale_to_surface", rva(0x4063B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const DECODED_WIDTH_RVA: usize = rva(0x519124); // g_decodedWidth
const DECODED_HEIGHT_RVA: usize = rva(0x519128); // g_decodedHeight
const DECODED_STRIDE_RVA: usize = rva(0x51913C); // g_decodedStride
const DECODED_PIXELS_RVA: usize = rva(0x519140); // g_decodedPixels
/// The work variables from `g_rescaleStepX` on: four steps, two weights, two counters,
/// the components.
const WORK_LEN: usize = 0x24;
const WEIGHT_X: usize = 0x10;
const WEIGHT_Y: usize = 0x14;
const COLUMNS_LEFT: usize = 0x18;
const ROWS_LEFT: usize = 0x1C;
const COMPONENTS: usize = 0x20;
/// `g_surface` (0x4F4998), twelve dwords, and the fields used here.
const SURFACE_RVA: usize = rva(0x4F4998);
const SURFACE_LEN: usize = 0x30;
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
        let format = (field(FORMAT) as *const PixelFormat).read_unaligned();
        let words = field(BITS) == 0x10;

        let (step_x, step_x_fraction) = step(from_width, width);
        let (step_y, step_y_fraction) = step(from_height, height);
        game_write(
            STEP_X_RVA,
            [step_x, step_x_fraction, step_y, step_y_fraction],
        );

        let mut out = field(PIXELS) as *mut u8;
        let (mut y, mut y_fraction) = (0u32, 0u32);
        for _ in 0..height {
            let weight_y = (y_fraction >> 24) as i32;
            game_write(STEP_X_RVA + WEIGHT_Y, weight_y as u32);
            let row = row_address(y);
            let below = if y.wrapping_add(1) != from_height {
                row_address(y.wrapping_add(1))
            } else {
                row
            };
            let (mut x, mut x_fraction) = (0u32, 0u32);
            for _ in 0..width {
                let weight_x = (x_fraction >> 24) as i32;
                game_write(STEP_X_RVA + WEIGHT_X, weight_x as u32);
                let at = |row: u32, part: u32| {
                    (row.wrapping_add(x.wrapping_mul(4)).wrapping_add(part) as *const u8).read()
                        as i32
                };
                for part in 0..3u32 {
                    let upper = blend(at(row, part), at(row, part + 4), weight_x);
                    let lower = blend(at(below, part), at(below, part + 4), weight_x);
                    game_write(
                        STEP_X_RVA + COMPONENTS + part as usize,
                        blend(upper, lower, weight_y) as u8,
                    );
                }
                let pixel = components_to_pixel(&format, game_read(STEP_X_RVA + COMPONENTS));
                if words {
                    (out as *mut u16).write_unaligned(pixel as u16);
                    out = out.add(2);
                } else {
                    (out as *mut u32).write_unaligned(pixel);
                    out = out.add(4);
                }
                let (fraction, carry) = x_fraction.overflowing_add(step_x_fraction);
                x_fraction = fraction;
                x = x.wrapping_add(step_x).wrapping_add(carry as u32);
            }
            out = out.wrapping_add(field(ROW_GAP) as usize);
            let (fraction, carry) = y_fraction.overflowing_add(step_y_fraction);
            y_fraction = fraction;
            y = y.wrapping_add(step_y).wrapping_add(carry as u32);
        }
        game_write(STEP_X_RVA + COLUMNS_LEFT, 0u32);
        game_write(STEP_X_RVA + ROWS_LEFT, 0u32);
        Ok(())
    }
}

// ---- hook ----

/// What a run writes besides the surface's pixels.
fn work_regions() -> Vec<Region> {
    vec![
        (game_addr(STEP_X_RVA) as u32, WORK_LEN),
        (game_addr(ROW_TABLE_RVA) as u32, ROW_TABLE_ROOM as usize * 4),
        at(SCRATCH_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { rescale_to_surface(game_addr(SURFACE_RVA) as *const u32) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    // The surface's pixels, as far as its description says they go.
    let surface = game_addr(SURFACE_RVA) as *const u32;
    let field = |offset: usize| unsafe { surface.byte_add(offset).read_unaligned() };
    let bytes = if field(BITS) == 0x10 { 2 } else { 4 };
    let row = (field(WIDTH) as usize)
        .saturating_mul(bytes)
        .saturating_add(field(ROW_GAP) as usize);
    let mut regions = work_regions();
    if field(PIXELS) != 0 && row.saturating_mul(field(HEIGHT) as usize) < 0x100_0000 {
        regions.push((field(PIXELS), row * field(HEIGHT) as usize));
    }
    dispatch_pixels(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

const LEN: usize = 0x1CD;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xA1, 0x40, 0x91, 0x51, 0x00, 0x8B, 0x1D];
/// `gfx::fill_row_table` and `gfx::components_to_pixel`.
const CALLS: [usize; 2] = [0x12, 0x177];
/// How often the code names a field of the surface.
const SURFACE_NAMED: usize = 10;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x4063B7)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The copy's surface becomes the test's.
    let mut surface = [0u32; SURFACE_LEN / 4];
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let real = game_addr(SURFACE_RVA) as u32;
        let mut patches = Vec::new();
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            if (real..real + SURFACE_LEN as u32).contains(&named) {
                patches.push((at, surface.as_mut_ptr() as u32 + (named - real)));
                at += 4;
            } else {
                at += 1;
            }
        }
        if patches.len() != SURFACE_NAMED {
            return t.skip("the original is not what the test was made from");
        }
        for (at, new) in patches {
            copy.patch_u32(at, new);
        }
    }
    let inputs = [
        at(DECODED_WIDTH_RVA),
        at(DECODED_HEIGHT_RVA),
        at(DECODED_STRIDE_RVA),
        at(DECODED_PIXELS_RVA),
    ];
    let work = work_regions();
    let saved = (capture(&inputs), capture(&work));
    // (decoded width, height, bytes between rows beyond the pixels; surface width,
    // height, bits, gap; which test format)
    for (from_width, from_height, extra, width, height, bits, gap, format) in [
        (8u32, 6u32, 0u32, 8u32, 6u32, 32u32, 0u32, 0usize),
        (8, 6, 0, 16, 12, 32, 8, 1),
        (8, 6, 8, 20, 9, 16, 0, 2),
        (40, 30, 4, 10, 7, 16, 6, 3),
        (7, 5, 0, 33, 31, 32, 4, 4),
        (1, 1, 0, 9, 9, 16, 2, 5),
        (64, 3, 12, 5, 40, 32, 0, 6),
        (3, 50, 0, 90, 2, 16, 0, 7),
    ] {
        let row_dwords = (from_width + extra / 4) as usize;
        let source: Vec<u32> = (0..row_dwords as u32 * from_height + 4)
            .map(|n| n.wrapping_mul(0x9E37_79B1) ^ n << 7)
            .collect();
        let row_bytes = (width * bits / 8 + gap) as usize;
        let mut target = vec![0u8; row_bytes * height as usize + 16];
        let before = Regs::SENTINEL;
        let mut run = |rust_side: bool| unsafe {
            game_write(DECODED_WIDTH_RVA, from_width);
            game_write(DECODED_HEIGHT_RVA, from_height);
            game_write(DECODED_STRIDE_RVA, from_width * 4 + extra);
            game_write(DECODED_PIXELS_RVA, source.as_ptr() as u32);
            restore(&work, &vec![0xAA; saved.1.len()]);
            target.fill(0x5A);
            let mut described = [0u32; SURFACE_LEN / 4];
            described[WIDTH / 4] = width;
            described[HEIGHT / 4] = height;
            described[BITS / 4] = bits;
            described[FORMAT / 4] = &TEST_FORMATS[format] as *const PixelFormat as u32;
            described[PIXELS / 4] = target.as_mut_ptr() as u32;
            described[ROW_GAP / 4] = gap;
            std::ptr::write_volatile(&raw mut surface, described);
            let ended = if rust_side {
                rescale_to_surface(&raw const surface as *const u32).is_err() as u32
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                !ro.preserved_except(&before, 0) as u32
            };
            // The row table holds addresses in the test's memory: as offsets.
            let mut left = capture(&work);
            for row in 0..ROW_TABLE_ROOM as usize {
                let at = WORK_LEN + row * 4;
                let address = u32::from_le_bytes(left[at..at + 4].try_into().unwrap());
                left[at..at + 4]
                    .copy_from_slice(&address.wrapping_sub(source.as_ptr() as u32).to_le_bytes());
            }
            (ended, left, target.clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!(
                "{from_width}x{from_height} to {width}x{height} at {bits} bits, format {format}: original ended {}, rust ended {}; first different pixel byte {:?}; first different work byte {:?}",
                theirs.0,
                mine.0,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
    restore(&inputs, &saved.0);
    restore(&work, &saved.1);
    t.finish()
}
