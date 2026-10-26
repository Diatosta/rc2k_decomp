//! `menu::add_player_times` (0x4A25AD): each counted player's (`dword_755984`) stage time
//! (+0x88) added to the total (+0x8C) when it took part (+0xD0 1), else both 3600000.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a player count of 0 or past 4 or a missing player,
//! before anything. Self-test: the original and the Rust version on the game's players,
//! the flags by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn add_player_times(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::event::added_time`.
    for p in counted_players()? {
        let entered = get(p + 0xD0);
        // The times are read only for a player that took part, as the original.
        let (time, total) = if entered == 1 {
            (get(p + 0x88), get(p + 0x8C))
        } else {
            (0, 0)
        };
        put_all(p, &rc2k_core::menu::event::added_time(entered, time, total));
    }
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { add_player_times(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
