//! `effects::scale_setting` (0x4C08BB): `WindscreenFlakeState.scaled_setting` = the settings' whole number +0x2750 by
//! `EdgeSettings.factor` (`fild; fmul; fstp`).
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::shapes_frame`. Refuses null settings (the original reads through 0).
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::{BadInput, Refused};
use crate::math::x87;

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
        let marks = effects.settings()?;
        let n = marks.setting_to_scale.get() as i32;
        fl.state
            .scaled_setting
            .set(x87::int_times(n, &masks.edge_settings.factor));
    }
    Ok(())
}
