//! `menu::next_record_kind` (0x470D08): the kind (first dword) of the record after the
//! current one (`g_recordTable`, section `section`, index `index` + 1), kept in
//! `dword_7559EC`.
//!
//! In:  nothing
//! Out: eax the kind; the rest kept
//!
//! Callers: `saves::load_savegame`, the event drivers. `records::find_record`'s Rust
//! version. Refuses a section or index past 63 or a section the table does not hold,
//! before anything.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const SECTION_RVA: usize = rva(0x75595C); // section
const INDEX_RVA: usize = rva(0x755958); // index
const RECORDS_RVA: usize = rva(0x735576); // g_recordTable
const KIND_RVA: usize = rva(0x7559EC); // dword_7559EC

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn next_record_kind(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let section: u32 = unsafe { game_read(SECTION_RVA) };
    let index: u32 = unsafe { game_read(INDEX_RVA) };
    let table = unsafe { Raw::new(game_addr(RECORDS_RVA) as *const u8) };
    // Converted: `rc2k_core::menu::record_kinds::next_record_kind`.
    g.eax = rc2k_core::menu::record_kinds::next_record_kind(&table, section, index)?;
    unsafe { game_write(KIND_RVA, g.eax) };
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { next_record_kind(r) }?))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
