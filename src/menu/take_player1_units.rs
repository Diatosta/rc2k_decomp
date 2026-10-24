//! `menu::take_player1_units` (0x467EA8): the game's units from player 1's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `g_isImperialUnits` = player 1's `metric_units` (+0x178; despite the name the variable
//! holds 1 for metric). Called at the start of each kind of event (`sub_482382` and more).
//! Self-test: the game's player 1 with values of the test's, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::menu::player::{Player, player_at};

pub fn take_player1_units() -> Result<(), Refused> {
    let p = player_at(0);
    if p == 0 {
        return Err(BadInput("no player 1 in g_players").into());
    }
    // g_isImperialUnits
    crate::strings::units::set_units(unsafe { (*(p as *const Player)).metric_units });
    Ok(())
}
