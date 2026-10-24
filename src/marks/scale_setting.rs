//! `marks::scale_setting` (0x4C08BB): `flt_8BC25E` = the settings' whole number +0x2750 by
//! `flt_8BC39E` (`fild; fmul; fstp`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1148`. Refuses null settings (the original reads through 0). Self-test:
//! settings of the test's with several numbers, the original and the Rust version
//! compared on the result and esi; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_setting_plain`, through
//! `rc2k_core::marks::scales::scaled_setting` (plain floats; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::math::x87;

const NUMBER_AT: u32 = 0x2750;
const FACTOR_RVA: usize = rva(0x8BC39E); // flt_8BC39E
const RESULT_RVA: usize = rva(0x8BC25E); // flt_8BC25E

/// The Rust version (see the header).
pub unsafe fn scale_setting() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let n = ((settings + NUMBER_AT) as *const i32).read_unaligned();
        game_write(RESULT_RVA, x87::int_times(n, game_read(FACTOR_RVA)));
    }
    Ok(())
}
