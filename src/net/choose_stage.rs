//! `net::choose_stage` (0x418347): the lobby's stage item (the host's): the stage screen
//! (`menu::select_stage_loop`, the Escape flag `escape_pressed` 0 first); unless it was left
//! with Escape, this player's record given the stage (+4 from the chosen stage) and its
//! direction (+5 from the stage's direction); then the background again (`net::show_background`).
//!
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::choose_car::ESCAPE_PRESSED;
use crate::net::store::NetVars;
use crate::net::store::PlayerRecord;
use crate::race::owner::Race;

/// The stage screen; the lobby loop's leftovers it carries (see `net::choose_car`) are all 0
/// here.
pub fn choose_stage(race: &Race, vars: &NetVars) -> Result<(), Refused> {
    ESCAPE_PRESSED.set(false);
    {
        run_screen(
            &crate::menu::select_stage_loop::SCREEN,
            &Entry::new(crate::owners::boundary()),
        )
    }?;
    if !ESCAPE_PRESSED.is_set() {
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
