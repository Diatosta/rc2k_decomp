//! `effects::scale_setting`: `WindscreenFlakeState.scaled_setting` = the settings' whole number
//! `setting_to_scale` by `EdgeSettings.factor`.
//!
//! Works on the effects settings.
//!
//! Refuses null settings.

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
        fl.state.borrow_mut().scaled_setting = x87::int_times(n, &masks.edge_settings.factor);
    }
    Ok(())
}
