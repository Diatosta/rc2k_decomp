//! `race::shadow_angles` (0x4581F3): the shadow's points ordered round the farthest one
//! (see `race/car_shadow.rs`): it swapped into the first record, each angle made relative
//! to its (plus 2 pi below 0) and times 65536, sorted by `sub_4582AC` (the game's quicksort
//! of 0x20-byte records, covered here; its recursion a list of returns).
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `call` follows in `sub_4575C1`)
//!
//! Refuses (before anything) no points or more than 0x40. Shadow mode: by state.
//! Self-test: point records of the test's (angles round, repeated, below the first), the
//! farthest one at each place; all put back.
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
    Ok(all_regs(&unsafe { shadow_angles(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
