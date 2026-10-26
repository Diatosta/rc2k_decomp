//! `race::car_light_modes` (0x455B70): each race car's lights mode in +0x6FC (see
//! `race/frame_draw.rs`): `dword_63DBA4` the drawn view's car; all eight blocks' +0x6FC 0;
//! the drawn cars with lights get their nibbles by the shown car's state, `dword_606274`,
//! `dword_606270` and `dword_6238B4`.
//!
//! In:  nothing
//! Out: ebx the race cars (8 with none), esi the last block (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x4557D6), each view drawn. Refuses (before anything) a null
//! block among the eight or more than eight cars.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_light_modes(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
