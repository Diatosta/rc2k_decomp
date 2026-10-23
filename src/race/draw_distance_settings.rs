//! `race::draw_distance_settings` (0x43932A): the draw distance's level (`dword_60AC84`,
//! held to 0 ..= 4 by sign) picks the values of six tables: `dword_60A87C` (the grid's
//! reach, at least 1 by sign), `dword_60930C`, `dword_609358`, `dword_609308`,
//! `dword_609320`, `dword_60AC80`.
//!
//! In:  nothing
//! Out: ebx the level, eax the last value (a `lea` follows)
//!
//! See `race/frame_draw.rs`. Caller: `race::fog_table`. Shadow mode: by state. Self-test:
//! the game's tables, levels -1 .. 5 and 0x80000000; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { draw_distance_settings() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
