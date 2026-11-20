//! Stretches or shrinks the decoded picture to the picture's size, smoothly, in the picture buffer.
//! The decoded picture (decoded width x height dwords, rows the decoded row gap bytes apart) is
//! first copied to memory of its own (on the list `Rescaler::copy_memory`), and its rows are listed
//! with `fill_row_table`. Then for every pixel of picture width x height, written to the picture
//! buffer: its place in the copy as a whole number and a 32-bit fraction, both stepped by decoded
//! size / picture size; of the fraction the top eight bits weigh the pixel and its right neighbour,
//! and the row and the row below (the last row is its own row below). Three bytes of each pixel are
//! made that way; the fourth is left as it was. The memory is given back. The right neighbour of a
//! row's last pixel is the next row's first, and for the last row four bytes past the copy. Leaves
//! the steps (`step_x` .. `step_y_fraction`), the last weights (`weight_x`, `weight_y`), and 0 in
//! the two counters (`columns_left`, `rows_left`). Refuses sizes of 0, a decoded height above the
//! row table's room, and null pointers.

use crate::errors::{BadInput, Refused};
use crate::gfx::fill_row_table::ROW_TABLE_ROOM;
use crate::gfx::picture::{has_picture, read_bytes, with_picture};
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::memory::session::{PlainBytes, PlainDword};

/// The rescaler's variables (`gfx::rescale_picture`, `gfx::rescale_to_surface`).
#[derive(rc2k_derive::Cells)]
pub struct Rescaler {
    /// `Rescaler.copy_memory.head`: the list the copy's memory is on (16 bytes).
    pub copy_memory: crate::memory::ListCells,
    /// Whole pixels a step across.
    pub step_x: PlainDword,
    /// The step's fraction.
    pub step_x_fraction: PlainDword,
    /// Whole rows a step down.
    pub step_y: PlainDword,
    /// The step's fraction.
    pub step_y_fraction: PlainDword,
    /// The weight across.
    pub weight_x: PlainDword,
    /// The weight down.
    pub weight_y: PlainDword,
    /// The columns left (left at 0).
    pub columns_left: PlainDword,
    /// The rows left (left at 0).
    pub rows_left: PlainDword,
    /// The pixel's three parts (`rescale_to_surface`), the fourth byte left as it was.
    pub components: PlainBytes<4>,
}

impl Rescaler {
    pub const fn new() -> Self {
        Rescaler {
            copy_memory: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
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

pub(super) fn rescale_picture(
    rescaler: &Rescaler,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
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
    // A failed allocation cannot come back: the allocator raises its own error. The copy is Rust's
    // memory, counted on the list as the game's block was; its rows are read in it, not through the
    // row table.
    let list = &rescaler.copy_memory;
    let row_bytes = from_width.wrapping_mul(4);
    let mut copy = crate::memory::owned::counted(list, from_width.wrapping_mul(from_height) << 2)?;
    let gap: u32 = pictures.decoder.row_gap.get();
    let mut source = 0usize;
    for row in copy
        .chunks_exact_mut(row_bytes as usize)
        .take(from_height as usize)
    {
        row.copy_from_slice(&read_bytes(
            &gfx.pictures.buffers,
            from,
            source,
            row_bytes as usize,
        )?);
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
