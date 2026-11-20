//! Works a pixel format's layout out from its masks (`PixelFormat`: the layout, the four masks, the
//! number).
//! If each of the four masks is empty or one run of bits: the layout is four bytes, lowest first
//! the masks' top bits' numbers less 7, 15, 23 and 31 (an empty mask counts as 0xFF;
//! `gfx::top_bit`), with the sign bit set: the rotation counts the colour conversions use. If the
//! four masks together are one run as well, the format's number is looked up by that layout in the
//! table of six at `numbers` (number, layout: 1 .. 4 the 16-bit formats, 5 and 6 the 32-bit ones);
//! else, and when it is not there, the number is 0. If a mask has a gap: the layout is the number
//! of the top bit of all masks together, and the number 0.

use crate::gfx::formats::Formats;

const NUMBERS: usize = 6;
pub use rc2k_core::gfx::pixels::describe;

/// The layout and number of a format of `masks` (blue, green, red, alpha), with the number table.
pub fn describe_masks(formats: &Formats, masks: [u32; 4]) -> (u32, u32) {
    let numbers: [(u32, u32); NUMBERS] = std::array::from_fn(|n| {
        (
            formats.numbers[n].number.get(),
            formats.numbers[n].layout.get(),
        )
    });
    describe(masks, &numbers)
}

/// On a format record (layout, four masks, number).
pub fn describe_format<C: crate::memory::session::DwordLike>(formats: &Formats, format: &[C; 6]) {
    let (layout, number) = describe_masks(formats, std::array::from_fn(|n| format[1 + n].get()));
    format[0].set(layout);
    format[5].set(number);
}
