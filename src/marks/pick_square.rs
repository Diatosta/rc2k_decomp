//! `marks::pick_square` (0x4C0623): one step of the squares' own random (`dword_8BC256`,
//! times 0xC5 plus 0x5D) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 21 ..
//! 23) for `marks::spawn_square`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::squares_spawn` (twice). The call is `spawn_square`'s Rust version.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::errors::Refused;
use crate::marks::spawn_square::spawn_square;
use crate::marks::squares_store::SQUARE_STATE;

/// The Rust version (see the header).
pub fn pick_square() -> Result<(), Refused> {
    {
        let s = SQUARE_STATE
            .pick_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        SQUARE_STATE.pick_random.set(s);
        spawn_square(s & 0x7FF, (s >> 11) & 0x3FF, (s >> 21) & 7)?;
    }
    Ok(())
}
