//! `scene::unlock_frame` (0x44E851): the back surface unlocked (`gfx::unlock_surface`, the
//! platform's) and the kept pitch and pixels (`dword_5C5FB4`, `dword_5C5FB0`) 0.
//!
//! In:  nothing
//! Out: CF as the unlock leaves it; eax 0; the rest as the unlock leaves them
//!
//! Callers: `race::loading_bar_frame` and three of the game's. The unlock through `part` (1),
//! in the game the platform's call. Hook removed 2026-10-22 (direct calls): Rust callers run
//! its `entry` (`calls::call`, `routes.rs`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_write, rva};

const PLATFORM_RVA: usize = rva(0x405949); // gfx::unlock_surface

/// A call by place: 1 the platform's.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers (and flags) it leaves.
pub unsafe fn unlock_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    part(1, &mut g)?;
    g.eax = 0;
    unsafe {
        game_write(rva(0x5C5FB4), 0u32);
        game_write(rva(0x5C5FB0), 0u32);
    }
    Ok(g)
}

/// In the game: the platform's function at its address.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(PLATFORM_RVA, r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { unlock_frame(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
