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
use crate::marks::edges::EDGE_SETTINGS;
use crate::marks::squares_store::SQUARE_STATE;
use crate::math::x87;
use crate::pointers::GamePtr;

const NUMBER_AT: u32 = 0x2750;

/// The Rust version (see the header).
pub fn scale_setting() -> Result<(), Refused> {
    {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let n = GamePtr::new(settings + NUMBER_AT).read::<i32>("a mark's field")?;
        SQUARE_STATE
            .scaled_setting
            .set_bits(x87::int_times(n, EDGE_SETTINGS.factor.bits()));
    }
    Ok(())
}
