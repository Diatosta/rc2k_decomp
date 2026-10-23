//! `marks::pong_bat_back` (0x4B8215): the pong bat put back where it was before its move
//! (`dword_7686AC`, `dword_7686B0`) when it overlaps the ball (`marks::pong`'s four
//! compares).
//!
//! In:  nothing
//! Out: eax the last compare's status (& 0x700), or with the bat put back eax / ebx its
//!      place's bits
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

use crate::framework::{Regs, game_read, game_write};
use crate::marks::pong;
use rc2k_core::marks::pong::Back;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pong_bat_back(entry: &Regs) -> Regs {
    let mut g = *entry;
    let (over, status) = pong::overlap();
    g.eax = status;
    if over {
        unsafe {
            g.eax = game_read(pong::SAVED_BAT[0]);
            g.ebx = game_read(pong::SAVED_BAT[1]);
            game_write(pong::BAT[0], g.eax);
            game_write(pong::BAT[1], g.ebx);
        }
    }
    g
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::pong::Court::bat_back`
pub unsafe fn pong_bat_back_plain(entry: &Regs) -> Regs {
    let mut g = *entry;
    match pong::with_court(|c| c.bat_back()) {
        Back::Kept(status) => g.eax = status,
        Back::PutBack([x, y]) => (g.eax, g.ebx) = (x, y),
    }
    g
}
