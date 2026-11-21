//! The PNG header's fields: `decode_png` writes them from the file's header chunk and, without one
//! in the file, reads what the last file left.

use std::cell::Cell;

/// The header of the PNG being decoded.
pub struct PngHeader {
    /// The picture's width.
    pub width: Cell<u32>,
    /// The picture's height.
    pub height: Cell<u32>,
    /// The header's byte after the colour type (the compression method).
    pub compression: Cell<u8>,
    /// The colour type.
    pub colour_type: Cell<u32>,
}

impl PngHeader {
    /// The header.
    pub const fn new() -> Self {
        PngHeader {
            width: Cell::new(0),
            height: Cell::new(0),
            compression: Cell::new(0),
            colour_type: Cell::new(0),
        }
    }
}

#[cfg(test)]
impl PngHeader {
    /// Every value of the record, in field order: what a test keeps to put back.
    pub fn values(&self) -> Vec<u32> {
        let out = vec![
            self.width.get(),
            self.height.get(),
            self.colour_type.get(),
            u32::from(self.compression.get()),
        ];
        out
    }

    /// The values `values` returned put back.
    pub fn set_values(&self, values: &[u32]) {
        let mut values = values.iter().copied();
        let mut next = || values.next().expect("a value kept for every field");
        self.width.set(next());
        self.height.set(next());
        self.colour_type.set(next());
        self.compression.set(next() as u8);
    }
}
