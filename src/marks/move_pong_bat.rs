//! `marks::move_pong_bat` (0x4B82B0): the pong bat's place kept (`dword_7686AC`,
//! `dword_7686B0`), then `marks::pong_bat_back`. The keys' moves between (up with
//! `byte_4F6B94` and `byte_4F6B91`, down with `byte_4F6B84`, none with `byte_4F6B86`, held
//! to `flt_7691E6` .. ) are jumped over by the original's two `jmp short`s: the bat
//! never moves (section 16).
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
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

use crate::marks::pong;
use crate::marks::pong_bat_back::pong_bat_back;
use crate::marks::store::{game_read, game_write};

/// The Rust version (see the header).
pub unsafe fn move_pong_bat() {
    unsafe {
        game_write(pong::SAVED_BAT[0], game_read::<u32>(pong::BAT[0]));
        game_write(pong::SAVED_BAT[1], game_read::<u32>(pong::BAT[1]));
        pong_bat_back()
    }
}
