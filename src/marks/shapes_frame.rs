//! `marks::shapes_frame` (0x4C1148): `marks::copy_first_point`, `marks::tilt_shape`,
//! `marks::scale_setting`, `marks::run_shapes` (then `nullsub_19`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1270`. Refuses null settings before anything, and its steps' refusals
//! (all it may write is put back first).

use crate::errors::{BadInput, Refused};
use crate::marks::{copy_first_point, run_shapes, scale_setting, tilt_shape};

/// The Rust version (see the header).
pub fn shapes_frame() -> Result<(), Refused> {
    if crate::marks::setup::SETTINGS.pointer.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    {
        copy_first_point::copy_first_point();
        tilt_shape::tilt_shape();
        scale_setting::scale_setting()?;
        run_shapes::run_shapes()
    }
}
