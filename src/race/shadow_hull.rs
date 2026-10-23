//! `race::shadow_hull` (0x45836F): the shadow's points (in angle order) reduced to a
//! convex hull (see `race/car_shadow.rs`): the first copied after the last, each point
//! that does not turn left (64-bit cross products of whole numbers, `qword_620460`)
//! taken out, starting again, until none; `dword_643ACC` the count.
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `cmp` follows, not of them)
//!
//! Refuses (before anything) fewer than two points or more than 0x40. Shadow mode: by
//! state. Self-test: point records of the test's (squares, stars, lines); all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { shadow_hull(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
