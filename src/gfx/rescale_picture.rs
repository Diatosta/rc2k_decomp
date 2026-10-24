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
use crate::gfx::blit::Slot as BlitSlot;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::pcx::{DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, PIXELS_RVA, ROW_GAP_RVA};
use crate::gfx::picture::{HEIGHT_RVA, WIDTH_RVA, picture};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;

// ---- game data ----

const COPY_MEMORY_RVA: BlitSlot = BlitSlot::at(0x4F505C); // dword_4F505C: the list the copy's memory is on
pub const STEP_X_RVA: BlitSlot = BlitSlot::at(0x4F509C); // dword_4F509C: whole pixels a step
const STEP_X_FRACTION_RVA: BlitSlot = BlitSlot::at(0x4F50A0); // dword_4F50A0
const STEP_Y_RVA: BlitSlot = BlitSlot::at(0x4F50A4); // dword_4F50A4
const STEP_Y_FRACTION_RVA: BlitSlot = BlitSlot::at(0x4F50A8); // dword_4F50A8
const WEIGHT_X_RVA: BlitSlot = BlitSlot::at(0x4F50AC); // dword_4F50AC
const WEIGHT_Y_RVA: BlitSlot = BlitSlot::at(0x4F50B0); // dword_4F50B0
const COLUMNS_LEFT_RVA: BlitSlot = BlitSlot::at(0x4F50B4); // dword_4F50B4
const ROWS_LEFT_RVA: BlitSlot = BlitSlot::at(0x4F50B8); // dword_4F50B8

// ---- Rust version ----

// Converted: `rc2k_core::gfx::picture` (the arithmetic); here the copy, the row table and the
// work variables.

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
        let mut out = to;
        let work = rc2k_core::gfx::picture::rescale(
            &rc2k_core::gfx::picture::Sizes {
                from_width,
                from_height,
                width,
                height,
            },
            &|row, byte| (row_address(row).wrapping_add(byte) as *const u8).read(),
            &mut |parts| {
                std::ptr::copy_nonoverlapping(parts.as_ptr(), out, 3);
                out = out.add(4);
            },
        )
        .expect("the sizes were checked");
        game_write(STEP_X_RVA, work.step_x);
        game_write(STEP_X_FRACTION_RVA, work.step_x_fraction);
        game_write(STEP_Y_RVA, work.step_y);
        game_write(STEP_Y_FRACTION_RVA, work.step_y_fraction);
        game_write(WEIGHT_X_RVA, work.weight_x);
        game_write(WEIGHT_Y_RVA, work.weight_y);
        game_write(COLUMNS_LEFT_RVA, 0u32);
        game_write(ROWS_LEFT_RVA, 0u32);
        free_list(list)?;
        Ok(())
    }
}
