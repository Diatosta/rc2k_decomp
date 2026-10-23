//! `race::both_split_colours` (0x45B147): `race::split_colours` for the first view's block
//! into `dword_6D160C`, then the second's into `dword_6D170C`.
//!
//! In:  nothing
//! Out: esi the second block, edi 0x6D170C, the rest as the second call leaves them (a
//!      `retn` follows)
//!
//! See `race/frame_draw.rs`. Caller: `race::set_up_panel` (0x45AB54). Refuses (before
//! anything) as `split_colours` would for either view. Shadow mode: by state (both
//! buffers). Self-test: the game's view blocks with counts and values set, fills; all put
//! back (skipped without view blocks).
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
    Ok(all_regs(&unsafe { both_split_colours(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
