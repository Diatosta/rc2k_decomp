//! `race::countdown_lights` (0x45B008): one view's start lights, from the time since the
//! countdown began (+0x14 - `dword_6D1814`, held to 1500).
//!
//! In:  edi 0 or 0x10 (the view's fields at 0x6D1482 + edi: lit, fade, size, shift), esi
//!      the view block, ebp where the "on a whole step" flag goes
//! Out: as `countdown_lights` in `race/frame_draw.rs` gives (eax, ebx, ecx, edx by path)
//!
//! Steps: fade, shift and the flag 0 (size: `sub [..], 0`, unchanged); nothing more when
//! +0x1C is set or the time is negative; `dword_6D1810` = hundreds (at most 5), lit = the
//! hundreds (-1 from 6); the flag 1 on a whole hundred (with lit not -1), else the rest
//! x 2 taken from 0x100 for fade and size; with five lit, fade = 0x80 x (held 0 .. 0x100)
//! / 0x30 and shift = half what that took away. Callers: `race::both_countdown_lights`
//! (twice; the next is a `mov` / `retn`). A reading: the five lights before the start.
//! Shadow mode: by state. Self-test: a test block and flag cell, times around each step,
//! both edi; the game's fields put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { countdown_lights(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
