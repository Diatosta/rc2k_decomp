//! The surface-locked switch.

use std::cell::Cell;

/// The surface-locked switch.
pub struct Locked {
    /// 1 while the screen's surface is locked for drawing.
    pub surface: Cell<u32>,
}

impl Locked {
    pub const fn new() -> Self {
        Locked {
            surface: Cell::new(0),
        }
    }
}
