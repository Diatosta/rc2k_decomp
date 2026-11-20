//! `menu::sealed_player_time`: the current player's (`event.current_player`) `time`
//! sealed (`records::seal_time`) with the player's car number, the run kind and a byte given by the
//! caller.
//!
//! Refuses a current player past 3 or missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;
use crate::race::owner::Race;

/// `byte` is the low byte sealed; the sealed block's two dwords come out.
pub(super) fn sealed_player_time(
    menus: &Menus,
    race: &Race,
    byte: u32,
) -> Result<[u32; 2], Refused> {
    let screens = &menus.screens;
    let n: u32 = screens.event.read().current_player;
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let p = player(menus, n);
    let sealed = {
        crate::records::seal_time::seal_time(
            menus,
            race,
            p.get().time,
            [
                p.get().car_number as u8,
                race.shared.state.settings.run_kind.get() as u8,
                byte as u8,
            ],
        )
    };

    Ok(sealed)
}
