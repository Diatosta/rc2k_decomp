//! `race::sky_matrix` (0x42725E): `g_madeObjectMatrix` for the sky (a reading: object 1,
//! drawn by `sub_455540`): turned about y by -(`dword_609334` << 8), raised by
//! `dword_609318` << 8, scaled by (`dword_6241D0` << 4) / 16, times the view's turn (see
//! `race/frame_draw.rs`, with `scene::matrix`'s Rust versions).
//!
//! In:  nothing
//! Out: eax `dword_6241D0`, ebx its x 16, ecx 0, edx 0, esi `g_turnedMatrix`, edi
//!      `g_viewTurnMatrix`, ebp `g_madeObjectMatrix` (a `lea` follows)
//!
//! Caller: `sub_455540` (0x455569). Shadow mode: by state (the matrices it builds, the
//! place and size dwords, `dword_62400C`). Self-test: the angle, height, size and view turn
//! set by pattern; all put back.
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
    Ok(unsafe { sky_matrix() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
