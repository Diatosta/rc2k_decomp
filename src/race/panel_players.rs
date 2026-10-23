//! `race::panel_players` (0x45ACDB): the panel's car fields by players: unless
//! `dword_62389C` is set, `dword_6D141A` and `dword_6D144A` -1; with more than one player
//! also `dword_6D1426` and `dword_6D142E` -1, and `dword_6D141A` / `dword_6D144A` the two
//! view blocks' +0xFA4.
//!
//! In:  nothing
//! Out: eax the second block's +0xFA4 when written, else kept (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB40), the panel's set-up each
//! frame. The fields (unaligned, in a block at 0x6D14xx) are a reading. Shadow mode: by
//! state. Self-test: the game's view blocks' +0xFA4 and the variables set, each case of
//! the two tests; all put back (skipped without view blocks).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_62389C
// dword_63DB94

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { panel_players() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
