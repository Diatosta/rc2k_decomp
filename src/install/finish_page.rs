//! `install::finish_page` (0x414347): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 2, `byte_51BE21` becomes 5 and the answer 1 (the readme is shown).
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
use crate::install::installer::{LIST_ANSWER, gb, logic, pb};
use crate::race::viewer::regs_entry;

pub fn finish_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    r.set_carry(false);
    r.set_zero((r.ebx as i32) >= 0);
    if (r.ebx as i32) < 0 {
        let answer = gb(LIST_ANSWER) as u32;
        logic(&mut r, answer.wrapping_sub(2));
        // Converted: `rc2k_core::install::pages::shows_readme`.
        if rc2k_core::install::pages::shows_readme(answer as u8) {
            pb(0x51BE21, 5);
            pb(LIST_ANSWER, 1);
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(finish_page(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
