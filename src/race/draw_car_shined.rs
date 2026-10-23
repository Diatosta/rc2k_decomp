//! `race::draw_car_shined` (0x461669): `g_shine` for the car by `dword_623D80` and
//! `dword_60AD0C`, the car drawn (`race::draw_car`), `g_shine` put back (see
//! `race/car_draw.rs`).
//!
//! In:  esi = the car's block
//! Out: eax, edx the shine's product, then as `draw_car` leaves them (callers return)
//!
//! Callers: `race::draw_car_entry`, 0x455B38. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose call goes to a stand-in noting `g_shine`, against the
//! Rust version with the same: distances in and out of the range.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_car_shined(r, &mut shined_part) }?;
    Ok(())
}
