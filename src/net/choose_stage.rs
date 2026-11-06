//! `net::choose_stage` (0x418347): the lobby's stage item (the host's): the stage screen
//! (`menu::select_stage_loop`, the Escape flag `dword_755A3C` 0 first); unless it was left
//! with Escape, this player's record given the stage (+4 from `dword_75ACD4`) and its
//! direction (+5 from `dword_75ACF0`); then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::choose_car::ESCAPED;
use crate::net::store::PlayerRecord;

/// The Rust version. The stage screen is handed the lobby loop's leftovers in its carried
/// registers (see `net::choose_car`): all 0 here.
pub fn choose_stage() -> Result<(), Refused> {
    ESCAPED.set(0);
    { run_screen(&crate::menu::select_stage_loop::SCREEN, &Entry::default()) }?;
    if ESCAPED.get() != 1 {
        crate::net::owner::net().vars.own.head.set(
            PlayerRecord::STAGE,
            crate::race::owner::race().shared.state.settings.stage.get() as u8,
        );
        crate::net::owner::net().vars.own.head.set(
            PlayerRecord::DIRECTION,
            crate::race::owner::race()
                .shared
                .state
                .settings
                .direction
                .get() as u8,
        );
    }
    crate::net::show_background::show_background()?;
    Ok(())
}
