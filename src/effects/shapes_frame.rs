//! `effects::shapes_frame` (0x4C1148): `effects::copy_first_point`, `effects::tilt_shape`,
//! `effects::scale_setting`, `effects::run_shapes` (then `nullsub_19`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1270`. Refuses null settings before anything, and its steps' refusals
//! (all it may write is put back first).

use crate::effects::{copy_first_point, run_shapes, scale_setting, tilt_shape};
use crate::errors::{BadInput, Refused};

/// The Rust version (see the header).
pub fn shapes_frame() -> Result<(), Refused> {
    if crate::effects::setup::SETTINGS.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    {
        copy_first_point::copy_first_point();
        tilt_shape::tilt_shape();
        scale_setting::scale_setting()?;
        run_shapes::run_shapes()
    }
}
