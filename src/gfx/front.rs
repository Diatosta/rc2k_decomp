//! The DirectDraw front and back surface handles. The platform layer sets both to 1 ("there are
//! surfaces"); the clear functions test the front one.

use std::cell::Cell;

/// The surface handles.
pub struct Surfaces {
    /// The front surface, 0 with none.
    pub front: Cell<u32>,
    /// The back surface.
    pub back: Cell<u32>,
}

impl Surfaces {
    /// The handles.
    pub const fn new() -> Self {
        Surfaces {
            front: Cell::new(0),
            back: Cell::new(0),
        }
    }
}
