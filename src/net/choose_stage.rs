//! `net::choose_stage` (0x418347): the lobby's stage item (the host's): the stage screen
//! (`menu::select_stage_loop`, the Escape flag `dword_755A3C` 0 first); unless it was left
//! with Escape, this player's record given the stage (+4 from `dword_75ACD4`) and its
//! direction (+5 from `dword_75ACF0`); then the background again (`net::show_background`).
//!
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::choose_car::ESCAPED;
use crate::net::store::NetVars;
use crate::net::store::PlayerRecord;
use crate::race::owner::Race;

/// The Rust version. The stage screen is handed the lobby loop's leftovers in its carried
/// registers (see `net::choose_car`): all 0 here.
pub fn choose_stage(race: &Race, vars: &NetVars) -> Result<(), Refused> {
    ESCAPED.set(0);
    {
        run_screen(
            &crate::menu::select_stage_loop::SCREEN,
            &Entry::new(crate::menu::owner::menus()),
        )
    }?;
    if ESCAPED.get() != 1 {
        vars.own.head.set(
            PlayerRecord::STAGE,
            race.shared.state.settings.stage.get() as u8,
        );
        vars.own.head.set(
            PlayerRecord::DIRECTION,
            race.shared.state.settings.direction.get() as u8,
        );
    }
    crate::net::show_background::show_background(vars)?;
    Ok(())
}
