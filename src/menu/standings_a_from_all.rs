//! `menu::standings_a_from_all` (0x4A2B75): standings table A filled from the drivers
//! (`menu::standings_from_drivers`: emptied, each driver with its name), then each group's
//! (+0x90, 0 .. 3) counted players into the entries from 89, 139, 169 and 179 down
//! (`menu::copy_standing_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A94EC`. The steps are their Rust versions. Refuses a
//! player count of 0 or past 4, a player missing or without a name, and what the drivers'
//! fill refuses, before anything. Self-test: the original and the Rust version on the
//! game's tables, the players' groups by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`copy_standing_entry`, ebx the player's index).
unsafe fn players_by_group(
    entry: &Regs,
    first: u32,
    size: u32,
    firsts: [usize; 4],
) -> Result<(), Refused> {
    let players = counted_players()?;
    let groups: Vec<u32> = players.iter().map(|&p| get(p + 0x90)).collect();
    // Converted: `rc2k_core::menu::standings::by_group`.
    for (n, e) in rc2k_core::menu::standings::by_group(&groups, firsts) {
        unsafe {
            crate::menu::copy_standing_entry::copy_standing_entry(&Regs {
                edi: first + size * e as u32,
                esi: players[n],
                ebx: n as u32,
                ..*entry
            })
        }?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn standings_a_from_all(entry: &Regs) -> Result<Regs, Refused> {
    let players = counted_players()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    unsafe { crate::menu::standings_from_drivers::standings_from_drivers(entry) }?;
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    use rc2k_core::menu::standings::A_GROUP_FIRSTS;
    unsafe { players_by_group(entry, first, A_ENTRY, A_GROUP_FIRSTS) }?;
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { standings_a_from_all(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
