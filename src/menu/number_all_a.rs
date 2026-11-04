//! `menu::number_all_a` (0x4A2090): the entries of standings table A numbered 1, 2, .. in
//! order (+0x44; with `dword_75AB40` 2 only those with a group, +0x1C not 0), a
//! player's number also into its record (+0x150); the count into `dword_75597C`.
//!
//! In:  nothing
//! Out: ecx the count, edi the end entry; esi / eax the last player's record and number
//! less 1 (when one was numbered); the rest kept
//!
//! Callers: the event drivers. Refuses a table without its end entry or a player past 4 or missing,
//! before anything.
use crate::errors::Refused;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;

/// The Rust version (see the header); what it leaves in the registers as `Numbered`.
pub fn number_all_a() -> Result<Numbered, Refused> {
    let grouped = MENU_STATE.menus.class_mode.get() == 2;
    let which = Numbering::All { grouped };
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::Numbering;
    let numbered = number_table(Which::A, which, |p| &mut p.number_all_a)?;
    EVENT_STATE.count_a.set(numbered.last_number);
    Ok(numbered)
}
