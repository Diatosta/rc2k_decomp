//! `menu::clear_score_marks` (0x4A919A): the 50 best-score entries (`dword_7556D4`, 0x3C
//! each; see `saves::default_best_scores`) given +0x14 0 and +4 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a null table, before anything. Self-test: the
//! original and the Rust version on a table of the test's; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const TABLE_RVA: usize = rva(0x7556D4); // dword_7556D4
const ENTRIES: u32 = 50;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_score_marks(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_RVA) };
    if first == 0 {
        return Err(BadInput("no best-score table").into());
    }
    // Converted: `rc2k_core::menu::standings::BEST_MARKS`, stored in each entry.
    unsafe { table(first, 0x3C, ENTRIES) }.store_all(&rc2k_core::menu::standings::BEST_MARKS);
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_score_marks(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
