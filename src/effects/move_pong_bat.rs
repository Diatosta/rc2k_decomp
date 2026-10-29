//! `effects::move_pong_bat` (0x4B82B0): the pong bat's place kept (`dword_7686AC`,
//! `dword_7686B0`), then `effects::pong_bat_back`. The keys' moves between (up with
//! `byte_4F6B94` and `byte_4F6B91`, down with `byte_4F6B84`, none with `byte_4F6B86`, held
//! to `flt_7691E6` .. ) are jumped over by the original's two `jmp short`s: the bat
//! never moves (section 16).
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
//!
//! Caller: `effects::run_pong`. The call is its Rust version.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::pong::PONG;
use crate::effects::pong_bat_back::pong_bat_back;

/// The Rust version (see the header).
pub fn move_pong_bat() {
    PONG.saved_bat[0].set(PONG.bat[0].bits());
    PONG.saved_bat[1].set(PONG.bat[1].bits());
    pong_bat_back()
}
