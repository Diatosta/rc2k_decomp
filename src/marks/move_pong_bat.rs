//! `marks::move_pong_bat` (0x4B82B0): the pong bat's place kept (`dword_7686AC`,
//! `dword_7686B0`), then `marks::pong_bat_back`. The keys' moves between (up with
//! `byte_4F6B94` and `byte_4F6B91`, down with `byte_4F6B84`, none with `byte_4F6B86`, held
//! to `flt_7691E6` .. ) are jumped over by the original's two `jmp short`s: the bat
//! never moves (section 16).
//!
//! In:  nothing
//! Out: as `pong_bat_back` leaves them (eax / ebx the bat's place going in)
//!
//! Caller: `marks::run_pong`. The call is its Rust version. Self-test: the original and
//! the Rust version on balls and bats by lot; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `move_pong_bat_plain`, through
//! `rc2k_core::marks::pong::Court::move_bat` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, game_write};
use crate::marks::pong;
use crate::marks::pong_bat_back::pong_bat_back;
use rc2k_core::marks::pong::Back;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn move_pong_bat(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(pong::BAT[0]);
        g.ebx = game_read(pong::BAT[1]);
        game_write(pong::SAVED_BAT[0], g.eax);
        game_write(pong::SAVED_BAT[1], g.ebx);
        pong_bat_back(&g)
    }
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::pong::Court::move_bat`
pub unsafe fn move_pong_bat_plain(entry: &Regs) -> Regs {
    let mut g = *entry;
    let ([x, y], back) = pong::with_court(|c| c.move_bat());
    (g.eax, g.ebx) = (x, y);
    match back {
        Back::Kept(status) => g.eax = status,
        Back::PutBack([x, y]) => (g.eax, g.ebx) = (x, y),
    }
    g
}
