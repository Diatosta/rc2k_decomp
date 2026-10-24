//! `net::choose_stage` (0x418347): the lobby's stage item (the host's): the stage screen
//! (`menu::select_stage_loop`, the Escape flag `dword_755A3C` 0 first); unless it was left
//! with Escape, this player's record given the stage (+4 from `dword_75ACD4`) and its
//! direction (+5 from `dword_75ACF0`); then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::screen_loop::{Carried, run_screen};
use crate::net::choose_car::ESCAPED;
use crate::net::fill_own_player::{DIRECTION, STAGE};
use crate::net::lobby::{OWN, g, p, pb};

/// The Rust version. The stage screen is handed the lobby loop's leftovers in its carried
/// registers (see `net::choose_car`): all 0 here.
pub fn choose_stage() -> Result<(), Refused> {
    p(ESCAPED, 0);
    unsafe { run_screen(&crate::menu::select_stage_loop::SCREEN, &Carried::default()) }?;
    if g(ESCAPED) != 1 {
        pb(OWN + 4, g(STAGE) as u8);
        pb(OWN + 5, g(DIRECTION) as u8);
    }
    crate::net::show_background::show_background()?;
    Ok(())
}
