//! `menu::number_standings_a` (0x4A2055): the entries of group ebx in standings table A numbered
//! 1, 2, .. in order (+0x44), a player's number also into its record (+0x14C).
//!
//! In:  ebx the group
//! Out: ecx the group's count, edi the end entry; esi / eax the last player's record and
//! number less 1 (when one was numbered); the rest kept
//!
//! Callers: the event pages' code. Refuses a table without its end entry or a player past 4 or
//! missing, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header); what it leaves in the registers as `Numbered`.
pub fn number_standings_a(class: u32) -> Result<Numbered, Refused> {
    let which = Numbering::Group(class);
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::Numbering;
    let numbered = number_table(Which::A, which, |p| &mut p.number_standings_a)?;
    Ok(numbered)
}
