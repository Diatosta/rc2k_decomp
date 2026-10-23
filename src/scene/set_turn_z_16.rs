//! `scene::set_turn_z_16` (0x427072): the turning matrix about the z axis for an angle of
//! sixteen bits a turn.
//!
//! In:  edx = the angle, sixteen bits a turn
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! As `scene::set_turn_x`: the sine and cosine kept in `dword_624158` and `dword_62415C`,
//! and to `flt_5EA00C` (the matrix of `scene::set_turn_z`, whose angle is 32 bits a turn):
//! +0 the cosine, +4 the sine, +0x10 the sine negated, +0x14 the cosine. Three callers
//! (`race::get_mat_crash` and two more). Self-test: 41 angles on the game's matrix, put
//! back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_turn_z_16_plain`, through
//! `rc2k_core::scene::matrices::set_table_turn`; compared exactly as before (the earlier
//! version stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::matrix::{self};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::set_turn_z_16_plain(r.edx) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
