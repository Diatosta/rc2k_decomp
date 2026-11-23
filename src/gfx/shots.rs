//! The screenshot switch and its lock.

use std::cell::Cell;

pub struct Shots {
    /// 1 = screenshots on (the `screensnapshot` setting in `var\ral.cfg`).
    pub on: Cell<u32>,
    /// The lock a screenshot holds.
    pub locked_here: Cell<u32>,
}

impl Shots {
    pub const fn new() -> Self {
        Shots {
            on: Cell::new(0),
            locked_here: Cell::new(0),
        }
    }
}

/// The switch's first byte.
pub fn on_byte(gfx: &crate::gfx::owner::Gfx) -> u8 {
    gfx.shots.on.get() as u8
}
