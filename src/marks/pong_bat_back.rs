//! `marks::pong_bat_back` (0x4B8215): the pong bat put back where it was before its move
//! (`dword_7686AC`, `dword_7686B0`) when it overlaps the ball (`marks::pong`'s four
//! compares).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves: `run_pong` reads none after
//!      `move_pong_bat`, the Rust versions none at all)
//!
//! Caller: `marks::move_pong_bat`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::pong::{self, PONG};

/// The Rust version (see the header).
pub fn pong_bat_back() {
    let (over, _) = pong::overlap();
    if over {
        PONG.bat[0].set_bits(PONG.saved_bat[0].get());
        PONG.bat[1].set_bits(PONG.saved_bat[1].get());
    }
}
