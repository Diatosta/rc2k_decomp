//! `effects::shapes_frame` (0x4C1148): `effects::copy_first_point`, `effects::tilt_shape`,
//! `effects::scale_setting`, `effects::run_shapes` (then `nullsub_19`).
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::view_frame`. Refuses null settings before anything, and its steps' refusals
//! (all it may write is put back first).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::{copy_first_point, run_shapes, scale_setting, tilt_shape};
use crate::errors::{BadInput, Refused};
use crate::race::owner_shared::RaceShared;

pub fn shapes_frame(
    shared: &RaceShared,
    masks: &Masks,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if effects.setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    {
        copy_first_point::copy_first_point(drops, effects);
        tilt_shape::tilt_shape(drops);
        scale_setting::scale_setting(masks, &effects.windscreen_flakes, effects)?;
        run_shapes::run_shapes(shared, masks, drops, effects)
    }
}
