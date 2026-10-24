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

use crate::marks::pong;
use crate::marks::store::{game_read, game_write};

/// The Rust version (see the header).
pub unsafe fn pong_bat_back() {
    let (over, _) = pong::overlap();
    if over {
        unsafe {
            game_write(pong::BAT[0], game_read::<u32>(pong::SAVED_BAT[0]));
            game_write(pong::BAT[1], game_read::<u32>(pong::SAVED_BAT[1]));
        }
    }
}
