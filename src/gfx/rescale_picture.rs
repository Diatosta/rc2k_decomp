//! `gfx::rescale_picture` (0x4061D2): stretches or shrinks the decoded picture to the
//! picture's size, smoothly, in the picture buffer.
//!
//! The decoded picture (`g_decodedWidth` x `g_decodedHeight` dwords at `g_decodedPixels`,
//! rows `g_decodedRowGap` bytes apart) is first copied to memory of its own (the game's
//! allocator, on the list `Rescaler::copy_memory`), and its rows
//! are listed with `fill_row_table`. Then for every pixel of `g_pictureWidth` x
//! `g_pictureHeight`, written to `g_pictureBuffer`: its place in the copy as a whole number
//! and a 32-bit fraction, both stepped by decoded size / picture size; of the fraction the
//! top eight bits weigh the pixel and its right neighbour, and the row and the row below
//! (the last row is its own row below). Three bytes of each pixel are made that way; the
//! fourth is left as it was. The memory is given back.
//! The right neighbour of a row's last pixel is the next row's first, and for the last row
//! four bytes past the copy: this reads the same bytes, from memory it takes
//! from the same allocator.
//! One caller, the picture loader `gfx::load_picture`, when the decoded width is not the screen's.
//! Leaves the steps (`step_x` .. `step_y_fraction`), the last weights (`weight_x`,
//! `weight_y`), and 0 in the two counters (`columns_left`, `rows_left`).
//! Refuses sizes of 0 (the original divides by them or counts through all 32 bits), a
//! decoded height above the row table's room, and null pointers. The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::gfx::fill_row_table::ROW_TABLE_ROOM;
use crate::gfx::picture::{has_picture, read_bytes, with_picture};
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::session::{PlainBytes, PlainDword};

// ---- game data ----

/// The rescaler's variables (`gfx::rescale_picture`, `gfx::rescale_to_surface`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Rescaler {
    /// `Rescaler.copy_memory.head`: the list the copy's memory is on (16 bytes).
    pub copy_memory: crate::memory::ListCells,
    /// `0x4F506C` .. `0x4F509C`: nothing reads or writes them (a gap).
    pub _gap_10: [PlainDword; 12],
    /// `dword_4F509C`: whole pixels a step across.
    pub step_x: PlainDword,
    /// `dword_4F50A0`: the step's fraction.
    pub step_x_fraction: PlainDword,
    /// `dword_4F50A4`: whole rows a step down.
    pub step_y: PlainDword,
    /// `dword_4F50A8`: the step's fraction.
    pub step_y_fraction: PlainDword,
    /// `dword_4F50AC`: the weight across.
    pub weight_x: PlainDword,
    /// `dword_4F50B0`: the weight down.
    pub weight_y: PlainDword,
    /// `dword_4F50B4`: the columns left (left at 0).
    pub columns_left: PlainDword,
    /// `dword_4F50B8`: the rows left (left at 0).
    pub rows_left: PlainDword,
    /// `0x4F50BC`: the pixel's three parts (`rescale_to_surface`), the fourth byte left as it was.
    pub components: PlainBytes<4>,
}

impl Rescaler {
    /// The rescaler's variables; the first values are the image's.
    pub const fn new() -> Self {
        Rescaler {
            copy_memory: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
            _gap_10: [const { PlainDword::new(0) }; 12],
            step_x: PlainDword::new(0),
            step_x_fraction: PlainDword::new(0),
            step_y: PlainDword::new(0),
            step_y_fraction: PlainDword::new(0),
            weight_x: PlainDword::new(0),
            weight_y: PlainDword::new(0),
            columns_left: PlainDword::new(0),
            rows_left: PlainDword::new(0),
            components: PlainBytes::new([0; 4]),
        }
    }
}

// Converted: `rc2k_core::gfx::picture` (the arithmetic); here the copy, the row table and the
// work variables.

pub fn rescale_picture(rescaler: &Rescaler, gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    let (from_width, from_height): (u32, u32) =
        (pictures.decoder.width.get(), pictures.decoder.height.get());
    let (width, height): (u32, u32) = (pictures.picture.width.get(), pictures.picture.height.get());
    let from = pictures.decoder.pixels.get();
    if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
        return Err(BadInput("a picture without width or height to rescale").into());
    }
    if from_height > ROW_TABLE_ROOM || from == 0 || !has_picture(pictures) {
        return Err(BadInput("a decoded picture the rescaler cannot take").into());
    }
    // The original calls a failed allocation fatal error 0x80A01112, but the allocator
    // never comes back failed: it raises its own.
    // The copy is Rust's memory, counted on the list as the game's block
    // was; its rows are read in it, not through the row table (which the original
    // fills with the copy's rows and nothing reads before it is filled again).
    let list = &rescaler.copy_memory;
    let row_bytes = from_width.wrapping_mul(4);
    let mut copy = crate::memory::owned::counted(list, from_width.wrapping_mul(from_height) << 2)?;
    let gap: u32 = pictures.decoder.row_gap.get();
    let mut source = 0usize;
    for row in copy
        .chunks_exact_mut(row_bytes as usize)
        .take(from_height as usize)
    {
        row.copy_from_slice(&read_bytes(from, source, row_bytes as usize)?);
        source = source
            .wrapping_add(row_bytes as usize)
            .wrapping_add(gap as usize);
    }
    // The picture's pixels made in its buffer, four bytes apart, three of each written.
    let work = with_picture(pictures, |to| {
        let mut out = 0usize;
        rc2k_core::gfx::picture::rescale(
            &rc2k_core::gfx::picture::Sizes {
                from_width,
                from_height,
                width,
                height,
            },
            &|row, byte| {
                let at = (row as usize * row_bytes as usize).wrapping_add(byte as usize);
                match copy.get(at) {
                    Some(&b) => b,
                    None => crate::fatal::raise::end(
                        "gfx::rescale_picture",
                        BadInput("a pixel read past the picture's copy"),
                    ),
                }
            },
            &mut |parts| {
                crate::gfx::picture::put_or_end(to, out, &parts[..3]);
                out += 4;
            },
        )
        .expect("the sizes were checked")
    })?;
    rescaler.step_x.set(work.step_x);
    rescaler.step_x_fraction.set(work.step_x_fraction);
    rescaler.step_y.set(work.step_y);
    rescaler.step_y_fraction.set(work.step_y_fraction);
    rescaler.weight_x.set(work.weight_x);
    rescaler.weight_y.set(work.weight_y);
    rescaler.columns_left.set(0);
    rescaler.rows_left.set(0);
    free_list(list)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.get().rescale,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Rescaler, step_x), 0x40);
        assert_eq!(offset_of!(Rescaler, rows_left), 0x5c);
        assert_eq!(offset_of!(Rescaler, components), 0x60);
    }
}
