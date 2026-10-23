//! For self-tests of functions whose variables are the game's all through a frame (the
//! work copy `g_car` and its neighbours): a `Mirror` of some of the game's ranges, filled
//! from the game's own contents (so that constants have their values), and copies of the
//! originals whose every reference into those ranges is pointed at the mirror (the number
//! of references checked). The Rust cores reach their variables through `Mirror::a`, which
//! for `Mirror::game()` is the game's own address.

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
