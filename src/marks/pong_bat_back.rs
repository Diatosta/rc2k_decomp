//! `marks::pong_bat_back` (0x4B8215): the pong bat put back where it was before its move
//! (`dword_7686AC`, `dword_7686B0`) when it overlaps the ball (`marks::pong`'s four
//! compares).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves: `run_pong` reads none after
//!      `move_pong_bat`, the Rust versions none at all)
//!
//! Caller: `marks::move_pong_bat`. Self-test: the original and the Rust version on balls
//! and bats by lot; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `pong_bat_back_plain`, through
//! `rc2k_core::marks::pong::Court::bat_back` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
