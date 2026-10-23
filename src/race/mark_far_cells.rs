//! `race::mark_far_cells` (0x454A5B): `g_courseGrid`'s cells with their top bits cleared,
//! then each cell turned by `g_viewTurnMatrix` and marked (bit 31) when it is out of sight
//! or too far (see `race/frame_draw.rs`; x87 throughout, compared as the original's
//! `fcomp` / `fnstsw` / `shr ah, 1`).
//!
//! In:  nothing
//! Out: eax and ecx the last cell's column and row less the half, ebx 0x400, esi
//!      `g_viewTurnMatrix`, edi past the grid (a `retn` follows)
//!
//! Writes `flt_63F180` / `flt_63F184` (the turned cell), `dword_62400C` (its length) and
//! `dword_624020` (`dword_63F17C` as a float) on the way. Caller:
//! `race::prepare_course_view` (0x4548DD). Shadow mode: by state. Self-test: the grid with
//! patterns (top bits set and not), halves 4 and 16, limits 16 and 3, on the game's view
//! turn; all put back.
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
    Ok(all_regs(&unsafe { mark_far_cells(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// dword_63E15C
// dword_63F17C
