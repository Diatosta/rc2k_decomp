//! `install::dxsetup_page` (0x41432F): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 1, `g_runDxSetup` is set so DirectX's set-up runs after the game.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of its last test
//! Caller `install::run_page`. Shadow mode compares.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{LIST_ANSWER, RUN_DXSETUP, gb, logic, p};
use crate::race::viewer::regs_entry;

pub fn dxsetup_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    r.set_carry(false);
    r.set_zero((r.ebx as i32) >= 0);
    if (r.ebx as i32) < 0 {
        let answer = gb(LIST_ANSWER) as u32;
        logic(&mut r, answer.wrapping_sub(1));
        // Converted: `rc2k_core::install::pages::runs_dxsetup`.
        if rc2k_core::install::pages::runs_dxsetup(answer as u8) {
            p(RUN_DXSETUP, 1);
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(dxsetup_page(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
