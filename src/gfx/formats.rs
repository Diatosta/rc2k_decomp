//! The pixel formats' work variables (format numbers and the component format). The row
//! table follows (`ROW_TABLE`).

use crate::memory::session::PlainDword;
use std::cell::Cell;

pub struct Formats {
    /// The pixel layouts' (number, layout) table: 1 .. 4 the 16-bit formats, 5 and 6 the 32-bit
    /// ones.
    pub numbers: [(u32, u32); 6],
    pub component_format: [PlainDword; 6],
    /// The bytes the pixel packers leave behind (kept as the game's work area; never read).
    pub scratch: Cell<[u8; 4]>,
}

impl Formats {
    pub const fn new() -> Self {
        Formats {
            numbers: [
                (1, 0xe0f8fbfd),
                (2, 0xe0f7fafd),
                (3, 0xf0f7fafd),
                (4, 0xf0f4f8fc),
                (5, 0xe0000000),
                (6, 0x80000000),
            ],
            component_format: [const { PlainDword::new(0) }; 6],
            scratch: Cell::new([0; 4]),
        }
    }
}
