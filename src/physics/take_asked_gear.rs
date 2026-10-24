//! `physics::take_asked_gear` (0x43C1E3): `g_car` +0x238 (`unk_71A210`) 0; the gear
//! asked for, bits 8 .. 11 of +0x468 (`unk_71A440`) as a signed nibble, through
//! `physics::allowed_gear` into `dword_614C40`; then `physics::change_gear`.
//!
//! In:  nothing
//! Out: as `change_gear` leaves them; no flags read (the caller returns)
//!
//! Caller: `sub_43C15C`. Refuses what the two calls refuse; what was written by then is put
//! back.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::physics::allowed_gear::allowed_gear;
use crate::physics::change_gear::change_gear;

/// The Rust version.
// Converted: `rc2k_core::physics::gearbox::asked_gear`.
pub unsafe fn take_asked_gear() -> Result<(), Refused> {
    let run = || -> Result<(), Refused> {
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
