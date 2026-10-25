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
use crate::gfx::decoder::DECODER;
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::picture::picture;
use crate::gfx::picture_store::PICTURE;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::session::{Bytes, Dword, Shared};

// ---- game data ----

/// IDA's address of the rescaler's variables.
pub const START: usize = 0x4F505C;
/// Their length.
pub const LEN: usize = 0x4F50C0 - START;

/// The rescaler's variables (`gfx::rescale_picture`, `gfx::rescale_to_surface`).
#[repr(C)]
pub struct Rescaler {
    /// `dword_4F505C`: the list the copy's memory is on (16 bytes).
    pub copy_memory: Shared<MemoryList>,
    /// `0x4F506C` .. `0x4F509C`: nothing reads or writes them (a gap).
    pub _gap_10: [Dword; 12],
    /// `dword_4F509C`: whole pixels a step across.
    pub step_x: Dword,
    /// `dword_4F50A0`: the step's fraction.
    pub step_x_fraction: Dword,
    /// `dword_4F50A4`: whole rows a step down.
    pub step_y: Dword,
    /// `dword_4F50A8`: the step's fraction.
    pub step_y_fraction: Dword,
    /// `dword_4F50AC`: the weight across.
    pub weight_x: Dword,
    /// `dword_4F50B0`: the weight down.
    pub weight_y: Dword,
    /// `dword_4F50B4`: the columns left (left at 0).
    pub columns_left: Dword,
    /// `dword_4F50B8`: the rows left (left at 0).
    pub rows_left: Dword,
    /// `0x4F50BC`: the pixel's three parts (`rescale_to_surface`), the fourth byte left as it was.
    pub components: Bytes<4>,
}

/// The rescaler's variables; the first values are the image's.
pub static RESCALE: Rescaler = Rescaler {
    copy_memory: Shared::new(MemoryList {
        head: 0,
        total: 0,
        cursor: 0,
        remaining: 0,
    }),
    _gap_10: [const { Dword::new(0) }; 12],
    step_x: Dword::new(0),
    step_x_fraction: Dword::new(0),
    step_y: Dword::new(0),
    step_y_fraction: Dword::new(0),
    weight_x: Dword::new(0),
    weight_y: Dword::new(0),
    columns_left: Dword::new(0),
    rows_left: Dword::new(0),
    components: Bytes::new([0; 4]),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x4F505C).
pub fn home() -> usize {
    &RESCALE as *const Rescaler as usize
}

// ---- Rust version ----

// Converted: `rc2k_core::gfx::picture` (the arithmetic); here the copy, the row table and the
// work variables.

pub unsafe fn rescale_picture() -> Result<(), Refused> {
    unsafe {
        let (from_width, from_height): (u32, u32) = (DECODER.width.get(), DECODER.height.get());
        let (width, height): (u32, u32) = (PICTURE.width.get(), PICTURE.height.get());
        let (from, to) = (DECODER.pixels.get(), picture());
        if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
            return Err(BadInput("a picture without width or height to rescale").into());
        }
        if from_height > ROW_TABLE_ROOM || from == 0 || to.is_null() {
            return Err(BadInput("a decoded picture the rescaler cannot take").into());
        }
        // The original calls a failed allocation fatal error 0x80A01112, but the allocator
        // never comes back failed: it raises its own.
        let list = RESCALE.copy_memory.as_ptr();
        let copy = allocate(list, from_width.wrapping_mul(from_height) << 2)? as u32;
        let gap: u32 = DECODER.row_gap.get();
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
        RESCALE.step_x.set(work.step_x);
        RESCALE.step_x_fraction.set(work.step_x_fraction);
        RESCALE.step_y.set(work.step_y);
        RESCALE.step_y_fraction.set(work.step_y_fraction);
        RESCALE.weight_x.set(work.weight_x);
        RESCALE.weight_y.set(work.weight_y);
        RESCALE.columns_left.set(0);
        RESCALE.rows_left.set(0);
        free_list(list)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&RESCALE, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Rescaler, step_x), 0x4F509C - START);
        assert_eq!(offset_of!(Rescaler, rows_left), 0x4F50B8 - START);
        assert_eq!(offset_of!(Rescaler, components), 0x4F50BC - START);
        assert_eq!(crate::moved::place(START), RESCALE.copy_memory.addr());
        assert_eq!(crate::moved::place(0x4F50BC), RESCALE.components.addr());
    }
}
