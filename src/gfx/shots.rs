//! The screenshot switch, its lock and the file name's template.

use crate::memory::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells)]
pub struct Shots {
    /// 1 = screenshots on (the `screensnapshot` setting in `var\ral.cfg`).
    pub on: PlainDword,
    /// The lock a screenshot holds.
    pub locked_here: PlainDword,
    /// The screenshot file's name, "000.png" and its NUL (the number's three digits are counted up
    /// for each file).
    pub name: PlainBytes<8>,
}

impl Shots {
    pub const fn new() -> Self {
        Shots {
            on: PlainDword::new(0),
            locked_here: PlainDword::new(0),
            name: PlainBytes::new(*b"000.png\0"),
        }
    }
}

/// The switch's first byte.
pub fn on_byte(gfx: &crate::gfx::owner::Gfx) -> u8 {
    gfx.shots.on.get() as u8
}
