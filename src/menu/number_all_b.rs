//! `menu::number_all_b` (0x4A5A98): the entries of standings table B numbered 1, 2, .. in
//! order (+0x70; with `dword_75AB40` 2 only those with a group, +0x10 not 0), a
//! player's number also into its record (+0x148).
//!
//! In:  nothing
//! Out: ecx the count, edi the end entry; esi / eax the last player's record and number
//! less 1 (when one was numbered); the rest kept
//!
//! Callers: the event drivers. Refuses a table without its end entry or a player past 4 or missing,
//! before anything.
use crate::errors::Refused;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;

/// The Rust version (see the header); what it leaves in the registers as `Numbered`.
pub fn number_all_b() -> Result<Numbered, Refused> {
    let grouped = MENU_STATE.menus.class_mode.get() == 2;
    let which = Numbering::All { grouped };
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::Numbering;
    let numbered = number_table(Which::B, which, |p| &mut p.number_all_b)?;
    Ok(numbered)
}
