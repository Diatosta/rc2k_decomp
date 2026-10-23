//! `physics::reset_current_car` (0x43FE18): `physics::reset_car_grades` on the work
//! copy (`lea esi, g_car`).
//!
//! In:  nothing
//! Out: esi `g_car`; eax 0, ecx 0, edi `g_car` + 0x368; no flags read (`jmp` next)
//!
//! Caller: `sub_43BA8E` (0x43BA97, when `dword_71A234` is 2), under `sub_43EB5D`.
//! Self-test: a copy of the original whose `g_car` is a test block and whose call goes
//! to a copy of `reset_car_grades` on the same block.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ESI, Regs, game_addr, rva};
use crate::physics::reset_car_grades::reset_car_grades;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const CAR_RVA: usize = rva(0x719FD8); // g_car

/// The Rust version.
pub unsafe fn reset_current_car() -> Left {
    let car = game_addr(CAR_RVA) as u32;
    unsafe { reset_car_grades(car) }.with(ESI, car)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_current_car() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
