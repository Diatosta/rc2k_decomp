//! `scene::lock_frame` (0x44E837): the back surface locked (`gfx::lock_surface`, the
//! platform's) and its pitch and pixels kept for the drawing (`dword_4F4988` into
//! `dword_5C5FB4`, `dword_4F4984` into `dword_5C5FB0`).
//!
//! In:  nothing
//! Out: CF as the lock leaves it; eax the pixels; the rest as the lock leaves them
//!
//! Callers: `race::loading_bar_frame` and three of the game's (0x442675, 0x4435A0,
//! 0x4435C2). The lock through `part` (1), in the game the platform's call. Hooked through a private
//! copy (a call at the entry).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

const PLATFORM_RVA: usize = rva(0x4058AD); // gfx::lock_surface

/// The Rust version (see the header): the registers (and flags) it leaves.
pub unsafe fn lock_frame(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    crate::scene::lock_frame::game_part(1, &mut g)?;
    unsafe {
        game_write(rva(0x5C5FB4), game_read::<u32>(rva(0x4F4988)));
        g.eax = game_read(rva(0x4F4984));
        game_write(rva(0x5C5FB0), g.eax);
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
    let left = unsafe { lock_frame(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
