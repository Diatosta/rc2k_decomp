//!
//! In:  nothing
//! Out: as the platform's call leaves them (CF read by the callers)
//!
//! Callers: `race::loading_bar_frame` and the game's (0x4395FB, 0x439605, 0x4426A0,
//! 0x4435DD).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, rva};

const PLATFORM_RVA: usize = rva(0x4057D1); // gfx::show_frame

/// The Rust version (see the header): the registers (and flags) it leaves.
pub unsafe fn present_frame(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    crate::scene::present_frame::game_part(1, &mut g)?;
    Ok(g)
}

/// In the game: the platform's function at its address.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(PLATFORM_RVA, r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { present_frame(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
