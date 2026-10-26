//! `race::car_lights` (0x455CFF): with `dword_621480` set, each race car's lights when they
//! are on (+0x6FC bit 5): its depth entry (`race::add_car_depth_entry`), then its flares
//! (`race::car_light_flares`), inside `pusha` (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: ebx the race cars; the rest kept (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x455858), each view drawn. Refuses (before anything) no cars or
//! more than eight, a null block, a depth list without room for all the cars' entries.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_lights(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
