//! The picture decoders' and glyph builder's variables (the builder's flags, the decoded picture's
//! layout, the glyph pixel format, the decoded palette). The PCX decoding's own variables (`PCX`),
//! the sheet table's (`gfx::build_glyphs::SHEET`) and the glyph tables' (`TABLES`) are separate;
//! the glyph builder reads and writes this block too, as the sheets of glyphs are PCX pictures.

use crate::session::PlainDword;

/// What the PCX and PNG readers lay out and the glyph builder works with.
#[derive(rc2k_derive::Cells)]
pub struct Decoder {
    /// How the sheet is read and its glyphs built.
    pub builder_flags: PlainDword,
    /// `template_`: the builder's file name template. Not used: nothing reads it, so the template
    /// is not kept here.
    pub template: PlainDword,
    /// `number`: the sheet's number in the template.
    pub number: PlainDword,
    /// The decoded picture's width, cut to what fits.
    pub width: PlainDword,
    pub height: PlainDword,
    /// The bits of a decoded pixel, 8 or 32.
    pub pixel_bits: PlainDword,
    /// The bytes of a decoded pixel.
    pub pixel_bytes: PlainDword,
    pub row_bytes: PlainDword,
    /// A row's dwords.
    pub row_dwords: PlainDword,
    /// From one row to the next.
    pub stride: PlainDword,
    /// Where the rows go.
    pub pixels: PlainDword,
    /// The stride less a row's bytes.
    pub row_gap: PlainDword,
    /// The pixel layout a sheet of components is coded in (a pixel format record by handle:
    /// `gfx::pixel_format::layout`).
    pub format: PlainDword,
    pub backdrop: PlainDword,
    pub transparent: PlainDword,
    /// 256 dwords.
    pub palette: [PlainDword; 256],
}

impl Decoder {
    /// The decoder's variables (all zero).
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
