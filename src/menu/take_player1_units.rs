//! `menu::take_player1_units` (0x467EA8): the game's units from player 1's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `g_isImperialUnits` = player 1's `metric_units` (+0x178; despite the name the variable holds 1
//! for metric). Called at the start of each kind of event (`sub_482382` and more).

use crate::errors::{BadInput, Refused};
use crate::menu::player::{Player, player_at};

pub fn take_player1_units() -> Result<(), Refused> {
    let p = player_at(0);
    if p == 0 {
        return Err(BadInput("no player 1 in g_players").into());
    }
    // g_isImperialUnits
    crate::strings::units::set_units(
        crate::pointers::GamePtr::new(p)
            .read::<Player>("player 1")?
            .metric_units,
    );
    Ok(())
}
