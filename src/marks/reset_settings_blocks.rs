//! `marks::reset_settings_blocks` (0x4C9A6D): in `init_marks`' step 3, the eight blocks
//! of `g_marksSettings` reset.
//!
//! In:  nothing
//! Out: ebp the settings, esi the first block, eax, ebx, ecx its three copied floats
//!
//! See `marks/blocks.rs`. One caller, `sub_4C3B01`. Self-test: the original and the Rust
//! version on settings of the test's, the game's pointer put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::blocks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_settings_blocks(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
