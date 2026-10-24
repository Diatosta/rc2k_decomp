//! `marks::view_frame` (0x4C1270): `marks::keep_vectors`, `marks::turn_kept_vectors`,
//! `marks::scale_kept_vectors`, `marks::view_step`, `marks::aim_vector`,
//! `marks::shapes_frame`, `marks::run_splines`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4D14B4`. Refuses null settings before anything, and its steps' refusals
//! (all it may write is put back first).

use crate::errors::{BadInput, Refused};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::store::game_read;
use crate::marks::{
    aim_vector, keep_vectors, run_splines, scale_kept_vectors, shapes_frame, view_step,
};

/// The Rust version (see the header).
pub unsafe fn view_frame() -> Result<(), Refused> {
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Err(BadInput("no marks settings").into());
    }
    unsafe {
        keep_vectors::keep_vectors();
        crate::marks::kept_turns::turn_kept_vectors();
        scale_kept_vectors::scale_kept_vectors();
        view_step::view_step();
        aim_vector::aim_vector();
        shapes_frame::shapes_frame()?;
        run_splines::run_splines()
    }
}
