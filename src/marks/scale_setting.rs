//! `marks::scale_setting` (0x4C08BB): `flt_8BC25E` = the settings' whole number +0x2750 by
//! `flt_8BC39E` (`fild; fmul; fstp`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1148`. Refuses null settings (the original reads through 0).
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::errors::{BadInput, Refused};
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const NUMBER_AT: u32 = 0x2750;
const FACTOR_RVA: Slot = Slot::at(0x8BC39E); // flt_8BC39E
const RESULT_RVA: Slot = Slot::at(0x8BC25E); // flt_8BC25E

/// The Rust version (see the header).
pub unsafe fn scale_setting() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let n = ((settings + NUMBER_AT) as *const i32).read_unaligned();
        game_write(RESULT_RVA, x87::int_times(n, game_read(FACTOR_RVA)));
    }
    Ok(())
}
