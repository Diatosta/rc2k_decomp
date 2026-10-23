//! `race::clear_car_marks` (0x46202A): the 0x40 dwords at `dword_5CD688` (the car's marks'
//! textures) 0 (see `race/car_draw.rs`).
//!
//! In:  nothing
//! Out: eax 0, ecx 0, edi past them
//!
//! Caller: `sub_4616AA`. Shadow mode: by state. Self-test: the buffer filled, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::car_draw::clear_car_marks;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { clear_car_marks() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
