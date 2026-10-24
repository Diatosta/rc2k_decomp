//! `marks::move_pong_bat` (0x4B82B0): the pong bat's place kept (`dword_7686AC`,
//! `dword_7686B0`), then `marks::pong_bat_back`. The keys' moves between (up with
//! `byte_4F6B94` and `byte_4F6B91`, down with `byte_4F6B84`, none with `byte_4F6B86`, held
//! to `flt_7691E6` .. ) are jumped over by the original's two `jmp short`s: the bat
//! never moves (section 16).
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
//!
//! Caller: `marks::run_pong`. The call is its Rust version.
//!
//! the x87 version stays for the Rust callers in unconverted code).

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
