//! The 3D reports switch (`reports.on`): Direct3D's and DirectDraw's failures are fatal while it is
//! 1; the film player, the mode trial and the texture allocation clear it around calls that may
//! fail.

use std::cell::Cell;

pub struct Reports {
    /// 1 while Direct3D's errors are reported (fatal).
    pub on: Cell<u32>,
}

impl Reports {
    pub const fn new() -> Self {
        Reports { on: Cell::new(1) }
    }
}
