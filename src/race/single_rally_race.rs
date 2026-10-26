//! `race::single_rally_race` (0x4843EC): a single rally's (and the other menus') race: the
//! track looked up, the settings and the players' records copied into the race's
//! variables, the race run (`race::race_from_menu`, the sound objects and the screen modes
//! round it), the results copied back. See `race/single_rally.rs`.
//!
//! In:  eax = 1 when the loading picture is not to be shown after, ebx = the kind (3:
//!      the rally's times copied first)
//! Out: every register preserved (`pusha` / `popa`)
//!
//! About 50 callers (the menus' races; each `cmp`s after, reading no flags). A `jnz` in
//! its first bytes: private copy of its body (0x1184 bytes, its 17 calls and 5 jumps to the
//! fatal stubs relocated), manual patch. Raises fatal 0x80739281 "Unknown Shaun TrakNum" (a
//! track not in `g_trakPakHeader`) from Rust; refuses a player's record missing (first).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::single_rally::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { single_rally_race(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
