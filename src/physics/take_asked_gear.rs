//! `physics::take_asked_gear` (0x43C1E3): `g_car` +0x238 (`unk_71A210`) 0; the gear
//! asked for, bits 8 .. 11 of +0x468 (`unk_71A440`) as a signed nibble, through
//! `physics::allowed_gear` into `dword_614C40`; then `physics::change_gear`.
//!
//! In:  nothing
//! Out: as `change_gear` leaves them; no flags read (the caller returns)
//!
//! Caller: `sub_43C15C`.
//! Refuses what the two calls refuse; what was written by then is put back.
//! Self-test: the original and the Rust version on the game's variables (the work car
//! and `dword_614C40` saved and put back): asked gears -8 .. 7, gears, bit patterns, the
//! two words of `allowed_gear`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::physics::allowed_gear::allowed_gear;
use crate::physics::change_gear::change_gear;
use crate::screens::harness::Left;

/// The Rust version.
// Converted: `rc2k_core::physics::gearbox::asked_gear`.
pub unsafe fn take_asked_gear() -> Result<Left, Refused> {
    let run = || -> Result<Left, Refused> {
        unsafe {
            game_write(rva(0x71A210), 0u32);
            let word: u32 = game_read(rva(0x71A440));
            let asked = rc2k_core::physics::gearbox::asked_gear(word);
            let gear = allowed_gear(asked)?;
            game_write(rva(0x614C40), gear);
            change_gear()
        }
    };

    run()
}
