//! `marks::view_frame` (0x4C1270): `marks::keep_vectors`, `marks::turn_kept_vectors`,
//! `marks::scale_kept_vectors`, `marks::view_step`, `marks::aim_vector`,
//! `marks::shapes_frame`, `marks::run_splines`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as `run_splines` leaves them
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
use crate::framework::{Regs, game_read};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::{
    aim_vector, keep_vectors, run_splines, scale_kept_vectors, shapes_frame, turn_kept_vectors,
    view_step,
};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

/// What its first five steps write (the last two put back their own).
fn early_regions() -> Vec<Region> {
    let mut r = keep_vectors::regions();
    r.extend(turn_kept_vectors::regions());
    r.extend(scale_kept_vectors::regions());
    r.extend(view_step::regions());
    r.extend(aim_vector::regions());
    r
}

/// What it may write.
pub fn regions() -> Vec<Region> {
    let mut r = early_regions();
    r.extend(shapes_frame::regions());
    r.extend(run_splines::regions(unsafe { game_read(SETTINGS_RVA) }));
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn view_frame(entry: &Regs) -> Result<Regs, Refused> {
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Err(BadInput("no marks settings").into());
    }

    unsafe {
        keep_vectors::keep_vectors();
        crate::marks::kept_turns::turn_kept_vectors();
        scale_kept_vectors::scale_kept_vectors();
        view_step::view_step();
        let g = aim_vector::aim_vector(entry);
        shapes_frame::shapes_frame(&g).and_then(|g| run_splines::run_splines(&g))
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { view_frame(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
