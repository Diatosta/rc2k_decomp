//! `menu::number_all_b` (0x4A5A98): the entries of standings table B numbered 1, 2, .. in
//! order (+0x70; with `dword_75AB40` 2 only those with a group, +0x10 not 0), a
//! player's number also into its record (+0x148).
//!
//! In:  nothing
//! Out: ecx the count, edi the end entry; esi / eax the last player's record and number
//!      less 1 (when one was numbered); the rest kept
//!
//! Callers: the event drivers. Refuses a table without its end entry or a player past 4
//! or missing, before anything. Self-test: the original and the Rust version on the
//! game's table and players, both modes; put back.
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

const MODE_RVA: usize = rva(0x75AB40); // dword_75AB40
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn number_all_b(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    let grouped = unsafe { game_read::<u32>(MODE_RVA) } == 2;
    let all = entries(first, B_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::{Numbering, number};
    let which = Numbering::All { grouped };
    let groups: Vec<u32> = all.iter().map(|&e| get(e + 0x10)).collect();
    for (&e, &group) in all.iter().zip(&groups) {
        let p = get(e + 4);
        if which.counts(group) && p != 0 && (p > 4 || player(p - 1) == 0) {
            return Err(BadInput("a standings player past 4 or missing").into());
        }
    }
    g.ecx = 0;
    for (&e, n) in all.iter().zip(number(&groups, which)) {
        let Some(n) = n else {
            continue;
        };
        g.ecx = n;
        if get(e + 4) != 0 {
            g.eax = get(e + 4) - 1;
            g.esi = player(g.eax);
            put(g.esi + 0x148, n);
        }
        put(e + 0x70, n);
    }
    g.edi = first + B_ENTRY * all.len() as u32;

    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { number_all_b(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
