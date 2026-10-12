//! `gfx::rescale_picture` (0x4061D2): stretches or shrinks the decoded picture to the
//! picture's size, smoothly, in the picture buffer.
//!
//! In:  nothing
//! Out: every register preserved
//! The decoded picture (`g_decodedWidth` x `g_decodedHeight` dwords at `g_decodedPixels`,
//! rows `g_decodedRowGap` bytes apart) is first copied to memory of its own (the game's
//! allocator, `memory::allocate`, on the list `dword_4F505C`), and its rows
//! are listed with `fill_row_table`. Then for every pixel of `g_pictureWidth` x
//! `g_pictureHeight`, written to `g_pictureBuffer`: its place in the copy as a whole number
//! and a 32-bit fraction, both stepped by decoded size / picture size; of the fraction the
//! top eight bits weigh the pixel and its right neighbour, and the row and the row below
//! (the last row is its own row below). Three bytes of each pixel are made that way; the
//! fourth is left as it was. The memory is given back.
//! The right neighbour of a row's last pixel is the next row's first, and for the last row
//! four bytes past the copy: the Rust version reads the same bytes, from memory it takes
//! from the same allocator.
//! One caller, the picture loader `sub_411863`, when the decoded width is not the screen's.
//! Leaves the steps in `dword_4F509C` .. `dword_4F50A8`, the last weights in `dword_4F50AC`
//! and `dword_4F50B0`, and 0 in the two counters `dword_4F50B4` and `dword_4F50B8`.
//! Refuses sizes of 0 (the original divides by them or counts through all 32 bits), a
//! decoded height above the row table's room, and null pointers. The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::harness::{check_pixels, dispatch_pixels};
use crate::gfx::pcx::{DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, PIXELS_RVA, ROW_GAP_RVA};
use crate::gfx::picture::{HEIGHT_RVA, PICTURE_DWORDS, WIDTH_RVA, picture};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::rescale_picture", rva(0x4061D2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const COPY_MEMORY_RVA: usize = rva(0x4F505C); // dword_4F505C: the list the copy's memory is on
pub const STEP_X_RVA: usize = rva(0x4F509C); // dword_4F509C: whole pixels a step
const STEP_X_FRACTION_RVA: usize = rva(0x4F50A0); // dword_4F50A0
const STEP_Y_RVA: usize = rva(0x4F50A4); // dword_4F50A4
const STEP_Y_FRACTION_RVA: usize = rva(0x4F50A8); // dword_4F50A8
const WEIGHT_X_RVA: usize = rva(0x4F50AC); // dword_4F50AC
const WEIGHT_Y_RVA: usize = rva(0x4F50B0); // dword_4F50B0
const COLUMNS_LEFT_RVA: usize = rva(0x4F50B4); // dword_4F50B4
const ROWS_LEFT_RVA: usize = rva(0x4F50B8); // dword_4F50B8

// ---- Rust version ----

/// One part towards another: `from + ((to - from) * weight >> 8)`, as the original computes
/// it in 32 bits.
pub fn blend(from: i32, to: i32, weight: i32) -> i32 {
    from.wrapping_add(to.wrapping_sub(from).wrapping_mul(weight) >> 8)
}

/// (whole part, fraction as 32 bits) of `a / b`.
pub fn step(a: u32, b: u32) -> (u32, u32) {
    (
        a / b,
        (((a % b) as u64) << 32).wrapping_div(b as u64) as u32,
    )
}

pub unsafe fn rescale_picture() -> Result<(), Refused> {
    unsafe {
        let (from_width, from_height): (u32, u32) =
            (game_read(DECODED_WIDTH_RVA), game_read(DECODED_HEIGHT_RVA));
        let (width, height): (u32, u32) = (game_read(WIDTH_RVA), game_read(HEIGHT_RVA));
        let (from, to) = (game_read::<u32>(PIXELS_RVA), picture());
        if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
            return Err(BadInput("a picture without width or height to rescale").into());
        }
        if from_height > ROW_TABLE_ROOM || from == 0 || to.is_null() {
            return Err(BadInput("a decoded picture the rescaler cannot take").into());
        }

        // The original calls a failed allocation fatal error 0x80A01112, but the allocator
        // never comes back failed: it raises its own.
        let list = game_addr(COPY_MEMORY_RVA) as *mut MemoryList;
        let copy = allocate(list, from_width.wrapping_mul(from_height) << 2)? as u32;
        let gap: u32 = game_read(ROW_GAP_RVA);
        let (mut source, mut target) = (from, copy);
        for _ in 0..from_height {
            for _ in 0..from_width {
                (target as *mut u32).write_unaligned((source as *const u32).read_unaligned());
                source = source.wrapping_add(4);
                target = target.wrapping_add(4);
            }
            source = source.wrapping_add(gap);
        }
        // The count was checked above.
        let _ = fill_row_table(copy, from_width << 2, from_height);

        let (step_x, step_x_fraction) = step(from_width, width);
        let (step_y, step_y_fraction) = step(from_height, height);
        game_write(STEP_X_RVA, step_x);
        game_write(STEP_X_FRACTION_RVA, step_x_fraction);
        game_write(STEP_Y_RVA, step_y);
        game_write(STEP_Y_FRACTION_RVA, step_y_fraction);

        let mut out = to;
        let (mut y, mut y_fraction) = (0u32, 0u32);
        for _ in 0..height {
            let weight_y = (y_fraction >> 24) as i32;
            game_write(WEIGHT_Y_RVA, weight_y as u32);
            let row = row_address(y);
            let below = if y.wrapping_add(1) != from_height {
                row_address(y.wrapping_add(1))
            } else {
                row
            };
            let (mut x, mut x_fraction) = (0u32, 0u32);
            for _ in 0..width {
                let weight_x = (x_fraction >> 24) as i32;
                game_write(WEIGHT_X_RVA, weight_x as u32);
                let at = |row: u32, part: u32| {
                    (row.wrapping_add(x.wrapping_mul(4)).wrapping_add(part) as *const u8).read()
                        as i32
                };
                for part in 0..3u32 {
                    let upper = blend(at(row, part), at(row, part + 4), weight_x);
                    let lower = blend(at(below, part), at(below, part + 4), weight_x);
                    out.add(part as usize)
                        .write(blend(upper, lower, weight_y) as u8);
                }
                out = out.add(4);
                let (fraction, carry) = x_fraction.overflowing_add(step_x_fraction);
                x_fraction = fraction;
                x = x.wrapping_add(step_x).wrapping_add(carry as u32);
            }
            let (fraction, carry) = y_fraction.overflowing_add(step_y_fraction);
            y_fraction = fraction;
            y = y.wrapping_add(step_y).wrapping_add(carry as u32);
        }
        game_write(COLUMNS_LEFT_RVA, 0u32);
        game_write(ROWS_LEFT_RVA, 0u32);

        free_list(list)?;
        Ok(())
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { rescale_picture() }?;
    Ok(Left::nothing())
}

/// What the function writes and leaves: the picture, and its eight work variables. (The
/// row table holds addresses in the copy's memory, which need not be the same twice.)
fn regions() -> Vec<Region> {
    let pixels =
        unsafe { game_read::<u32>(WIDTH_RVA) as usize * game_read::<u32>(HEIGHT_RVA) as usize };
    let mut regions = vec![(game_addr(STEP_X_RVA) as u32, 0x20)];
    if !picture().is_null() && pixels <= PICTURE_DWORDS {
        regions.push((picture() as u32, pixels * 4));
    }
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_pixels(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: small pictures of the test's own stretched and shrunk into the game's
// picture buffer, which is put back with the variables ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if picture().is_null() {
        return t.skip("the picture buffer is not allocated");
    }
    const PICTURE_BYTES: usize = 0x8000;
    let inputs = [
        at(DECODED_WIDTH_RVA),
        at(DECODED_HEIGHT_RVA),
        at(PIXELS_RVA),
        at(ROW_GAP_RVA),
        at(WIDTH_RVA),
        at(HEIGHT_RVA),
        (game_addr(STEP_X_RVA) as u32, 0x20),
        (
            game_addr(crate::gfx::fill_row_table::ROW_TABLE_RVA) as u32,
            ROW_TABLE_ROOM as usize * 4,
        ),
        (picture() as u32, PICTURE_BYTES),
    ];
    let saved = capture(&inputs);
    let original = ORIGINAL.load(Ordering::SeqCst);
    // (decoded width, height, gap in bytes, picture width, height)
    for (from_width, from_height, gap, width, height) in [
        (8u32, 6u32, 0u32, 8u32, 6u32),
        (8, 6, 0, 16, 12),
        (8, 6, 8, 20, 9),
        (40, 30, 4, 10, 7),
        (7, 5, 0, 33, 31),
        (1, 1, 0, 9, 9),
        (64, 3, 12, 5, 40),
        (3, 50, 0, 90, 2),
    ] {
        let row_dwords = (from_width + gap / 4) as usize;
        let source: Vec<u32> = (0..row_dwords as u32 * from_height + 4)
            .map(|n| n.wrapping_mul(0x9E37_79B1) ^ n << 7)
            .collect();
        let prepare = || unsafe {
            game_write(DECODED_WIDTH_RVA, from_width);
            game_write(DECODED_HEIGHT_RVA, from_height);
            game_write(PIXELS_RVA, source.as_ptr() as u32);
            game_write(ROW_GAP_RVA, gap);
            game_write(WIDTH_RVA, width);
            game_write(HEIGHT_RVA, height);
            std::ptr::write_bytes(game_addr(STEP_X_RVA) as *mut u8, 0xAA, 0x20);
            std::ptr::write_bytes(picture(), 0x5A, PICTURE_BYTES);
        };
        check_pixels(
            &mut t,
            &format!("{from_width}x{from_height} gap {gap} to {width}x{height}"),
            original,
            Regs::SENTINEL,
            &prepare,
            &[
                (game_addr(STEP_X_RVA) as u32, 0x20),
                (picture() as u32, PICTURE_BYTES),
            ],
            &rust,
        );
    }
    restore(&inputs, &saved);
    t.finish()
}
