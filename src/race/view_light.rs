//! `race::view_light` (0x455896): the light at the work car (`g_car` +0x134 .. into
//! `g_litPlace` ..), `race::light_colour_at`, then `dword_6415E4` = `dword_62DAD4` plus that
//! light byte by byte (through `dword_62400C`; three bytes without MMX, four with).
//!
//! In:  nothing
//! Out: eax the sum; the rest as `light_colour_at` leaves them (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x4557F4), each view drawn. Refuses as
//! `light_colour_at` (before anything).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { view_light(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
