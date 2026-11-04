//! The picture decoders' and glyph builder's variables (the builder's flags, the decoded
//! picture's layout, the glyph pixel format, the decoded palette), out of RAL.EXE's image
//! (phase 5; typed in phase 7, `notes/conversion.md`): IDA 0x519118 .. 0x519554. The PCX
//! decoding's own variables (`gfx::pcx::PCX`), the sheet table's (`gfx::build_glyphs::SHEET`)
//! and the glyph tables' (`gfx::glyph_tables::TABLES`) follow in the image; the glyph
//! builder reads and writes this block's too, so it is one block (they are one subsystem:
//! the sheets of glyphs are PCX pictures).

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x519118;
/// The block's length.
pub const LEN: usize = 0x519554 - START;

/// What the PCX and PNG readers lay out and the glyph builder works with.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Decoder {
    /// `g_glyphBuilderFlags` (`dword_519118`): how the sheet is read and its glyphs built.
    pub builder_flags: Dword,
    /// `template_` (`dword_51911C`): the builder's file name template. Not used: nothing
    /// reads it, so the template is not kept here.
    pub template: Dword,
    /// `number` (`dword_519120`): the sheet's number in the template.
    pub number: Dword,
    /// `g_decodedWidth` (`dword_519124`): the decoded picture's width, cut to what fits.
    pub width: Dword,
    /// `g_decodedHeight` (`dword_519128`).
    pub height: Dword,
    /// `dword_51912C`: the bits of a decoded pixel, 8 or 32.
    pub pixel_bits: Dword,
    /// `dword_519130`: the bytes of a decoded pixel.
    pub pixel_bytes: Dword,
    /// `g_decodedRowBytes` (`dword_519134`).
    pub row_bytes: Dword,
    /// `dword_519138`: a row's dwords.
    pub row_dwords: Dword,
    /// `g_decodedStride` (`dword_51913C`): from one row to the next.
    pub stride: Dword,
    /// `g_decodedPixels` (`dword_519140`): where the rows go.
    pub pixels: Dword,
    /// `g_decodedRowGap` (`dword_519144`): the stride less a row's bytes.
    pub row_gap: Dword,
    /// `g_glyphPixelFormat` (`dword_519148`): the pixel layout a sheet of components is
    /// coded in (a pixel format record by handle: `gfx::pixel_format::layout`).
    pub format: Dword,
    /// `g_sheetBackdrop` (`dword_51914C`).
    pub backdrop: Dword,
    /// `g_sheetTransparent` (`dword_519150`).
    pub transparent: Dword,
    /// `g_decodedPalette` (`dword_519154`): 256 dwords.
    pub palette: [Dword; 256],
}

/// The decoder's variables; the first values are the image's (all zero).
pub static DECODER: Decoder = Decoder {
    builder_flags: Dword::new(0),
    template: Dword::new(0),
    number: Dword::new(0),
    width: Dword::new(0),
    height: Dword::new(0),
    pixel_bits: Dword::new(0),
    pixel_bytes: Dword::new(0),
    row_bytes: Dword::new(0),
    row_dwords: Dword::new(0),
    stride: Dword::new(0),
    pixels: Dword::new(0),
    row_gap: Dword::new(0),
    format: Dword::new(0),
    backdrop: Dword::new(0),
    transparent: Dword::new(0),
    palette: [const { Dword::new(0) }; 256],
};

impl Decoder {
    /// The decoded palette's 256 entries.
    pub fn palette(&self) -> [u32; 256] {
        std::array::from_fn(|n| self.palette[n].get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&DECODER, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Decoder, width), 0x519124 - START);
        assert_eq!(offset_of!(Decoder, format), 0x519148 - START);
        assert_eq!(offset_of!(Decoder, palette), 0x519154 - START);
        assert_eq!(
            crate::moved::place(START),
            DECODER.builder_flags.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x519154),
            (crate::pointers::game_address(DECODER.palette.as_ptr())) as usize
        );
        assert_eq!(
            crate::moved::place(0x519140),
            DECODER.pixels.addr() as usize
        );
    }
}
