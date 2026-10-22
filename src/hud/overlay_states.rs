//! `hud::overlay_states` (0x45A943): the 2D overlay's render states: 0x15 = 4, 0x1B = 1,
//! 0x13 = 5, 0x14 = 6, 0x1C = 0.
//!
//! In:  nothing
//! Out: eax 0, ebx 0x1C; the rest as the state calls leave them
//!
//! Callers: `sub_45A893` (0x45A8A2), code at 0x4559AF, 0x4559C8. The states through
//! `part` (1 with eax the value, ebx the state), in the game `set_render_state`'s Rust
//! version.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;

/// A call by place: 1 `set_render_state`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::scene::render_states::OVERLAY`
pub unsafe fn overlay_states(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    for s in rc2k_core::scene::render_states::OVERLAY {
        [g.eax, g.ebx] = [s.value, s.state];
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `set_render_state`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    let (value, state) = (r.eax, r.ebx);
    unsafe { crate::scene::set_render_state::set_render_state(r, value, state) };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { overlay_states(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
