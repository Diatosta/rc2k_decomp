//! `marks::view_frame` (0x4C1270): `marks::keep_vectors`, `marks::turn_kept_vectors`,
//! `marks::scale_kept_vectors`, `marks::view_step`, `marks::aim_vector`,
//! `marks::shapes_frame`, `marks::run_splines`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4D14B4`. It begins with its call: a private copy (0x24 bytes, its seven
//! calls relocated) with a manual patch. Refuses null settings before anything, and its
//! steps' refusals (all it may write is put back first). Self-test: the game's settings
//! (skipped without them; outside a stage the splines' pointers may name freed memory, so
//! it runs only when `g_marksSplinePlace` is 0), compared on all it may write.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::marks::lines::SETTINGS_RVA;
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
