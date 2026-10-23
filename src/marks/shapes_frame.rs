//! `marks::shapes_frame` (0x4C1148): `marks::copy_first_point`, `marks::tilt_shape`,
//! `marks::scale_setting`, `marks::run_shapes` (then `nullsub_19`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as `run_shapes` leaves them
//!
//! Caller: `sub_4C1270`. It begins with its call: a private copy (0x1A bytes, its five
//! calls relocated) with a manual patch. Refuses null settings before anything, and its
//! steps' refusals (all it may write is put back first). Self-test: the game's settings
//! (skipped without them), its steps' inputs by lot; compared on all it may write.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::{copy_first_point, run_shapes, scale_setting, tilt_shape};
use crate::service::harness::Region;

/// What it may write.
pub fn regions() -> Vec<Region> {
    let mut r = copy_first_point::regions();
    r.extend(tilt_shape::regions());
    r.extend(scale_setting::regions());
    r.extend(run_shapes::regions());
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn shapes_frame(entry: &Regs) -> Result<Regs, Refused> {
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Err(BadInput("no marks settings").into());
    }

    unsafe {
        copy_first_point::copy_first_point();
        let g = tilt_shape::tilt_shape(entry);
        scale_setting::scale_setting(&g).and_then(|g| run_shapes::run_shapes(&g))
    }
}
