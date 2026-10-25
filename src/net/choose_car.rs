//! `net::choose_car` (0x4182FA): the lobby's car item: `dword_75598C` 0, the car screen
//! (`menu::select_car_loop` on `net::car_list`'s list, from player 1's car, the Escape flag
//! `dword_755A3C` 0 first); unless it was left with Escape, the car taken
//! (`menu::take_shown_car`) and this player's record given it (+1 the car, +2 its colours);
//! then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::screen_loop::{Carried, run_screen};
use crate::net::lobby::{g, p};
use crate::net::store::{NET, PlayerRecord};

const CLEARED: usize = 0x75598C; // dword_75598C
pub const ESCAPED: usize = 0x755A3C; // dword_755A3C: the screen was left with Escape
const CAR: usize = 0x755A54; // g_player1.car_number
const COLOURS: usize = 0x755A7C; // g_player1.field_30

/// The Rust version. The car screen starts from the car (eax) and the list of cars (esi);
/// the other registers it is handed are the lobby loop's leftovers (ebx the clock's tick,
/// ecx and edx scratch), which the screen overwrites or reads only as the x its first item
/// is measured from: 0 here, as the loop leaves ecx.
pub fn choose_car() -> Result<(), Refused> {
    p(CLEARED, 0);
    let cars = crate::net::car_list::car_list()?;
    p(ESCAPED, 0);
    let entry = Carried {
        arg: g(CAR),
        cars,
        ..Carried::default()
    };
    unsafe { run_screen(&crate::menu::select_car_loop::SCREEN, &entry) }?;
    if g(ESCAPED) != 1 {
        unsafe { crate::menu::take_shown_car::take_shown_car(cars) }?;
        NET.own.head.set(PlayerRecord::CAR, g(CAR) as u8);
        NET.own.head.set(PlayerRecord::COLOURS, g(COLOURS) as u8);
    }
    crate::net::show_background::show_background()?;
    Ok(())
}
