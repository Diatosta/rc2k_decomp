//! The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list
//! its memory is on). The texture grades are `gfx::texture_pixels::GRADING`.

use crate::memory::MemoryList;
use std::cell::Cell;

/// The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
/// components, before it is converted to the surface's format and drawn.
pub struct Picture {
    /// Flags of the picture; bits 8 .. 11 say what its pixels are (0x200 after a conversion to
    /// 16-bit pixels).
    pub flags: Cell<u32>,
    /// The picture's width in pixels.
    pub width: Cell<u32>,
    /// The picture's height in pixels.
    pub height: Cell<u32>,
    /// The bytes of one row (width * 4, halved when the picture is converted to 16-bit pixels);
    /// 0xFFFFFFFD while a picture is being loaded.
    pub row_bytes: Cell<u32>,
    /// The pointer to the buffer.
    pub buffer: Cell<u32>,
    /// The buffer's size in bytes (0x1D4C00: 800 x 600 dwords).
    pub size: Cell<u32>,
    /// `Picture.list.head`: the list the buffer's memory is on (16 bytes).
    pub list: crate::memory::PlainList,
}

impl Picture {
    pub const fn new() -> Self {
        Picture {
            flags: Cell::new(0),
            width: Cell::new(0),
            height: Cell::new(0),
            row_bytes: Cell::new(0),
            buffer: Cell::new(0),
            size: Cell::new(0),
            list: crate::memory::PlainList::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}

#[cfg(test)]
impl Picture {
    /// Every value of the record, in field order: what a test keeps to put back.
    pub fn values(&self) -> Vec<u32> {
        let mut out = vec![
            self.flags.get(),
            self.width.get(),
            self.height.get(),
            self.row_bytes.get(),
            self.buffer.get(),
            self.size.get(),
        ];
        out.extend({
            let l = self.list.get();
            [l.head, l.total, l.cursor, l.remaining]
        });
        out
    }

    /// The values `values` returned put back.
    pub fn set_values(&self, values: &[u32]) {
        let mut values = values.iter().copied();
        let mut next = || values.next().expect("a value kept for every field");
        self.flags.set(next());
        self.width.set(next());
        self.height.set(next());
        self.row_bytes.set(next());
        self.buffer.set(next());
        self.size.set(next());
        self.list.set(rc2k_core::memory::List {
            head: next(),
            total: next(),
            cursor: next(),
            remaining: next(),
        });
    }
}
