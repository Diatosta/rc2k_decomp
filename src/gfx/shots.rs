//! The screenshot switch, its lock and the file name's template.

use std::cell::Cell;

pub struct Shots {
    /// 1 = screenshots on (the `screensnapshot` setting in `var\ral.cfg`).
    pub on: Cell<u32>,
    /// The lock a screenshot holds.
    pub locked_here: Cell<u32>,
    /// The screenshot file's name, "000.png" (the number's three digits are counted up for each
    /// file).
    pub name: Cell<[u8; 7]>,
}

impl Shots {
    pub const fn new() -> Self {
        Shots {
            on: Cell::new(0),
            locked_here: Cell::new(0),
            name: Cell::new(*b"000.png"),
        }
    }
}

/// The switch's first byte.
pub fn on_byte(gfx: &crate::gfx::owner::Gfx) -> u8 {
    gfx.shots.on.get() as u8
}
