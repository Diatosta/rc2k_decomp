//! `menu::number_standings_b` (0x4A4A9D): the entries of group ebx in standings table B numbered
//! 1, 2, .. in order (+0x70), a player's number also into its record (+0x144).
//!
//! In:  ebx the group
//! Out: ecx the group's count, edi the end entry; esi / eax the last player's record and
//! number less 1 (when one was numbered); the rest kept
//!
//! Callers: the event pages' code. Refuses a table without its end entry or a player past 4 or
//! missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::memory::blocks::Place;
use crate::menu::standings::*;

/// The Rust version (see the header); what it leaves in the registers as `Numbered`.
pub fn number_standings_b(class: u32) -> Result<Numbered, Refused> {
    let mut last_number = 0;
    let mut last_player = None;
    let first = Place::Owned(TABLE_B_RVA.need("no standings table")?);
    let all = entries(first, B_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::{Numbering, number};
    let which = Numbering::Group(class);
    let groups: Vec<u32> = all
        .iter()
        .map(|&e| get_at(e.add(0x10)))
        .collect::<Result<_, _>>()?;
    for (&e, &group) in all.iter().zip(&groups) {
        let p = get_at(e.add(4))?;
        if which.counts(group) && p != 0 && (p > 4 || player(p - 1) == 0) {
            return Err(BadInput("a standings player past 4 or missing").into());
        }
    }
    for (&e, n) in all.iter().zip(number(&groups, which)) {
        let Some(n) = n else {
            continue;
        };
        last_number = n;
        if get_at(e.add(4))? != 0 {
            let index = get_at(e.add(4))? - 1;
            let record = player(index);
            put(record + 0x144, n)?;
            last_player = Some((index, record));
        }
        e.add(0x70).write(n, "a standings entry")?;
    }
    Ok(Numbered {
        last_number,
        last_player,
    })
}
