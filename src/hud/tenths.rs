//! `hud::tenths` (0x4B1D2B): six times of the HUD's record in tenths (signed `idiv` by
//! 10): +0x24, +0x2C, +0x20, +0x54, +0x58, +0x50 into `dword_75C074` .. `dword_75C088`.
//!
//! In:  ebp the HUD's record
//! Out: eax the last quotient, edx its remainder, ebx 10; the rest kept
//!
//! Caller: `hud::draw_hud`. Self-test: the original and the Rust version on records of the
//! test's, negative times among them.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const OUT_RVA: usize = rva(0x75C074); // dword_75C074 .. dword_75C088
const FIELDS: [u32; 6] = [0x24, 0x2C, 0x20, 0x54, 0x58, 0x50];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn tenths(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::hud::tenths::tenths`.
    let mut g = *entry;
    let times = FIELDS.map(|f| unsafe { ((g.ebp + f) as *const i32).read_unaligned() });
    let (tenths, remainder) = rc2k_core::hud::tenths::tenths(times);
    unsafe { game_write(OUT_RVA, tenths) };
    [g.eax, g.edx, g.ebx] = [tenths[5] as u32, remainder as u32, 10];
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { tenths(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
