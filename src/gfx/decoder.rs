//! The picture decoders' and glyph builder's variables (the builder's flags, the decoded picture's
//! layout, the glyph pixel format, the decoded palette), out of RAL.EXE's image: IDA 0x519118 ..
//! 0x519554. The PCX decoding's own variables (`PCX`), the sheet table's
//! (`gfx::build_glyphs::SHEET`) and the glyph tables' (`TABLES`) follow in the image; the glyph
//! builder reads and writes this block's too, so it is one block (they are one subsystem: the
//! sheets of glyphs are PCX pictures).

use crate::session::PlainDword;

/// What the PCX and PNG readers lay out and the glyph builder works with.
#[derive(rc2k_derive::Cells)]
pub struct Decoder {
    /// `g_glyphBuilderFlags` (`dword_519118`): how the sheet is read and its glyphs built.
    pub builder_flags: PlainDword,
    /// `template_` (`dword_51911C`): the builder's file name template. Not used: nothing
    /// reads it, so the template is not kept here.
    pub template: PlainDword,
    /// `number` (`dword_519120`): the sheet's number in the template.
    pub number: PlainDword,
    /// `g_decodedWidth` (`dword_519124`): the decoded picture's width, cut to what fits.
    pub width: PlainDword,
    /// `g_decodedHeight` (`dword_519128`).
    pub height: PlainDword,
    /// `dword_51912C`: the bits of a decoded pixel, 8 or 32.
    pub pixel_bits: PlainDword,
    /// `dword_519130`: the bytes of a decoded pixel.
    pub pixel_bytes: PlainDword,
    /// `g_decodedRowBytes` (`dword_519134`).
    pub row_bytes: PlainDword,
    /// `dword_519138`: a row's dwords.
    pub row_dwords: PlainDword,
    /// `g_decodedStride` (`dword_51913C`): from one row to the next.
    pub stride: PlainDword,
    /// `g_decodedPixels` (`dword_519140`): where the rows go.
    pub pixels: PlainDword,
    /// `g_decodedRowGap` (`dword_519144`): the stride less a row's bytes.
    pub row_gap: PlainDword,
    /// `g_glyphPixelFormat` (`dword_519148`): the pixel layout a sheet of components is
    /// coded in (a pixel format record by handle: `gfx::pixel_format::layout`).
    pub format: PlainDword,
    /// `g_sheetBackdrop` (`dword_51914C`).
    pub backdrop: PlainDword,
    /// `g_sheetTransparent` (`dword_519150`).
    pub transparent: PlainDword,
    /// `g_decodedPalette` (`dword_519154`): 256 dwords.
    pub palette: [PlainDword; 256],
}

impl Decoder {
    /// The decoder's variables; the first values are the image's (all zero).
    pub const fn new() -> Self {
        Decoder {
            builder_flags: PlainDword::new(0),
            template: PlainDword::new(0),
            number: PlainDword::new(0),
            width: PlainDword::new(0),
            height: PlainDword::new(0),
            pixel_bits: PlainDword::new(0),
            pixel_bytes: PlainDword::new(0),
            row_bytes: PlainDword::new(0),
            row_dwords: PlainDword::new(0),
            stride: PlainDword::new(0),
            pixels: PlainDword::new(0),
            row_gap: PlainDword::new(0),
            format: PlainDword::new(0),
            backdrop: PlainDword::new(0),
            transparent: PlainDword::new(0),
            palette: [const { PlainDword::new(0) }; 256],
        }
    }
}

impl Decoder {
    /// The decoded palette's 256 entries.
    pub fn palette(&self) -> [u32; 256] {
        std::array::from_fn(|n| self.palette[n].get())
    }
}
