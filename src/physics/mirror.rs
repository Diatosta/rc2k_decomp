//! The Rust cores reach their variables through `Mirror::a`, which for `Mirror::game()` is
//! the game's own address.

use crate::framework::{game_addr, rva};

/// The game's ranges, or a test's copies of them.
pub struct Mirror {
    ranges: Vec<(u32, u32)>,
    bases: Vec<u32>,
}

impl Mirror {
    /// The game's own variables.
    pub fn game() -> Self {
        Self {
            ranges: Vec::new(),
            bases: Vec::new(),
        }
    }

    /// The address of the variable IDA shows at `ida`.
    pub fn a(&self, ida: u32) -> u32 {
        for (k, &(lo, hi)) in self.ranges.iter().enumerate() {
            if (lo..hi).contains(&ida) {
                return self.bases[k] + (ida - lo);
            }
        }
        game_addr(rva(ida as usize)) as u32
    }

    pub unsafe fn get(&self, ida: u32) -> u32 {
        unsafe { (self.a(ida) as *const u32).read_unaligned() }
    }

    pub unsafe fn put(&self, ida: u32, v: u32) {
        unsafe { (self.a(ida) as *mut u32).write_unaligned(v) }
    }
}
