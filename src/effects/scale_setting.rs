//! `effects::scale_setting` (0x4C08BB): `flt_8BC25E` = the settings' whole number +0x2750 by
//! `flt_8BC39E` (`fild; fmul; fstp`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1148`. Refuses null settings (the original reads through 0).
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::{BadInput, Refused};
use crate::math::x87;

/// The Rust version (see the header).
pub fn scale_setting(
    masks: &Masks,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let settings: u32 = effects.setup.settings.is_set.get();
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let marks = crate::effects::setup::marks()?;
        let n = marks.unknown_b0.get() as i32;
        fl.state
            .scaled_setting
            .set_bits(x87::int_times(n, masks.edge_settings.factor.bits()));
    }
    Ok(())
}
