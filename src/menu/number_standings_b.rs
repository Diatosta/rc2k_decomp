//! `menu::number_standings_b` (0x4A4A9D): the entries of group ebx in standings table B numbered
//! 1, 2, .. in order (+0x70), a player's number also into its record (+0x144).
//!
//! In:  ebx the group
//! Out: ecx the group's count, edi the end entry; esi / eax the last player's record and
//!      number less 1 (when one was numbered); the rest kept
//!
//! Callers: the event pages' code. Refuses a table without its end entry or a player past
//! 4 or missing, before anything. Self-test: the original and the Rust version on the
//! game's table and players, groups 1 .. 3; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::menu::standings::*;

/// The Rust version (see the header); what it leaves in the registers as `Numbered`.
pub unsafe fn number_standings_b(class: u32) -> Result<Numbered, Refused> {
    let mut last_number = 0;
    let mut last_player = None;
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    let all = entries(first, B_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::{Numbering, number};
    let which = Numbering::Group(class);
    let groups: Vec<u32> = all.iter().map(|&e| get(e + 0x10)).collect();
    for (&e, &group) in all.iter().zip(&groups) {
        let p = get(e + 4);
        if which.counts(group) && p != 0 && (p > 4 || player(p - 1) == 0) {
            return Err(BadInput("a standings player past 4 or missing").into());
        }
    }
    for (&e, n) in all.iter().zip(number(&groups, which)) {
        let Some(n) = n else {
            continue;
        };
        last_number = n;
        if get(e + 4) != 0 {
            let index = get(e + 4) - 1;
            let record = player(index);
            put(record + 0x144, n);
            last_player = Some((index, record));
        }
        put(e + 0x70, n);
    }
    let end = first + B_ENTRY * all.len() as u32;
    Ok(Numbered {
        last_number,
        last_player,
        end,
    })
}
