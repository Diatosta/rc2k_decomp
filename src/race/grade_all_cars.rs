//! `race::grade_all_cars` (0x43FDFE): `race::grade_car_values` of the eight car blocks
//! (`g_carBlocks`), race init's last step.
//!
//! In:  nothing
//! Out: eax = the last block's last grade, ebx = 8, esi = the last block; the rest
//!      preserved; flags as `cmp 8, 8` leaves them (zero set, carry clear)
//!
//! See `race/car_grades.rs`. Callers: `sub_4462B9` (race init) and the code at 0x43FDDF
//! (after `sub_440A22` for each block; no function in IDA, no code reference). Shadow
//! mode: the Rust version, the grades put back, the original's, compared. Self-test: the
//! game's blocks' values set to floats round the thresholds, both versions, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ESI, Regs};
use crate::race::car_grades::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { grade_all_cars(r) };
    Ok(Left::carry(false)
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ESI, g.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
