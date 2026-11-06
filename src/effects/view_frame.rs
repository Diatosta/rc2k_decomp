//! `effects::view_frame` (0x4C1270): `effects::keep_vectors`, `effects::turn_kept_vectors`,
//! `effects::scale_kept_vectors`, `effects::view_step`, `effects::aim_vector`,
//! `effects::shapes_frame`, `effects::run_splines`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4D14B4`. Refuses null settings before anything, and its steps' refusals
//! (all it may write is put back first).

use crate::effects::owner::Effects;
use crate::effects::{
    aim_vector, keep_vectors, run_splines, scale_kept_vectors, shapes_frame, view_step,
};
use crate::errors::{BadInput, Refused};

/// The Rust version (see the header).
pub fn view_frame(effects: &Effects) -> Result<(), Refused> {
    if crate::effects::owner::effects().setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    {
        keep_vectors::keep_vectors();
        crate::effects::kept_turns::turn_kept_vectors();
        scale_kept_vectors::scale_kept_vectors();
        view_step::view_step();
        aim_vector::aim_vector();
        shapes_frame::shapes_frame(&effects.masks, &effects.windscreen_drops)?;
        run_splines::run_splines(&effects.paths)
    }
}
