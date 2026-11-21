//! The picture decoders' and glyph builder's variables (the builder's flags, the decoded picture's
//! layout, the glyph pixel format, the decoded palette). The PCX decoding's own variables (`PCX`),
//! the sheet table's (`gfx::build_glyphs::SHEET`) and the glyph tables' (`TABLES`) are separate;
//! the glyph builder reads and writes this block too, as the sheets of glyphs are PCX pictures.

use crate::memory::session::PlainDword;
use std::cell::Cell;

/// What the PCX and PNG readers lay out and the glyph builder works with.
pub struct Decoder {
    /// How the sheet is read and its glyphs built.
    pub builder_flags: Cell<u32>,
    /// `number`: the sheet's number in the template.
    pub number: Cell<u32>,
    /// The decoded picture's width, cut to what fits.
    pub width: Cell<u32>,
    pub height: Cell<u32>,
    /// The bits of a decoded pixel, 8 or 32.
    pub pixel_bits: Cell<u32>,
    /// The bytes of a decoded pixel.
    pub pixel_bytes: Cell<u32>,
    pub row_bytes: Cell<u32>,
    /// A row's dwords.
    pub row_dwords: Cell<u32>,
    /// From one row to the next.
    pub stride: Cell<u32>,
    /// Where the rows go.
    pub pixels: Cell<u32>,
    /// The stride less a row's bytes.
    pub row_gap: Cell<u32>,
    /// The pixel layout a sheet of components is coded in (a pixel format record by handle:
    /// `gfx::pixel_format::layout`).
    pub format: Cell<u32>,
    pub backdrop: Cell<u32>,
    pub transparent: Cell<u32>,
    /// 256 dwords.
    pub palette: [PlainDword; 256],
}

impl Decoder {
    /// The decoder's variables (all zero).
    pub const fn new() -> Self {
        Decoder {
            builder_flags: Cell::new(0),
            number: Cell::new(0),
            width: Cell::new(0),
            height: Cell::new(0),
            pixel_bits: Cell::new(0),
            pixel_bytes: Cell::new(0),
            row_bytes: Cell::new(0),
            row_dwords: Cell::new(0),
            stride: Cell::new(0),
            pixels: Cell::new(0),
            row_gap: Cell::new(0),
            format: Cell::new(0),
            backdrop: Cell::new(0),
            transparent: Cell::new(0),
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

#[cfg(test)]
impl Decoder {
    /// Every value of the record, in field order: what a test keeps to put back.
    pub fn values(&self) -> Vec<u32> {
        let mut out = vec![
            self.builder_flags.get(),
            self.number.get(),
            self.width.get(),
            self.height.get(),
            self.pixel_bits.get(),
            self.pixel_bytes.get(),
            self.row_bytes.get(),
            self.row_dwords.get(),
            self.stride.get(),
            self.pixels.get(),
            self.row_gap.get(),
            self.format.get(),
            self.backdrop.get(),
            self.transparent.get(),
        ];
        out.extend(self.palette().iter().copied());
        out
    }

    /// The values `values` returned put back.
    pub fn set_values(&self, values: &[u32]) {
        let mut values = values.iter().copied();
        let mut next = || values.next().expect("a value kept for every field");
        self.builder_flags.set(next());
        self.number.set(next());
        self.width.set(next());
        self.height.set(next());
        self.pixel_bits.set(next());
        self.pixel_bytes.set(next());
        self.row_bytes.set(next());
        self.row_dwords.set(next());
        self.stride.set(next());
        self.pixels.set(next());
        self.row_gap.set(next());
        self.format.set(next());
        self.backdrop.set(next());
        self.transparent.set(next());
        for entry in &self.palette {
            entry.set(next());
        }
    }
}
