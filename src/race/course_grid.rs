//! `race::course_grid` (0x45497E): the course records around the view, in a 32 x 32 grid
//! of record numbers at `dword_63E17C` (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: eax, ecx (and when the grid is made ebx the record count, esi the last record,
//!      edi past the grid) as it leaves them (a `call` follows)
//!
//! The half size is `race::hold_to_limit(dword_60A87C)` (`dword_71C6D4` is held to 4 .. 16
//! by `sub_4648B4`), the span twice that, so a span of 32 puts cells 0 .. 32 on each axis
//! into a grid 32 wide: column 32 lands on the next row's first, and row 32 past the grid
//! (onto `dword_63F17C` ..), as the original does (`notes/bugs.md`). Refuses (before
//! anything is written) no course records when the grid is to be made (the loop tests its
//! end after the first record). Shadow mode: by state (the grid and what lies past it to
//! 0x1084 bytes, the half, span, corner and the two cell numbers). Self-test: test records
//! (`g_courseRecords` pointed at them), corners, spans 8 and 32, the corner unchanged; all
//! put back.
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
    Ok(all_regs(&unsafe { course_grid(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// dword_60A87C
// dword_71C6D4
// dword_6062B0
// dword_6062B4
// g_courseRecords
// g_courseRecordCount
