//! `menu::apply_cheats` (0x480AC1): acts on the cheats that are switched on.
//!
//! In:  nothing
//! Out: every register preserved
//! The cheats are the bits of `g_cheats` (0x75870C); each that is set writes "Cheat n
//! (...)" to the game's log (`CHEATS` below, in this order):
//! - bits 0, 1, 13 .. 16 stay set and do nothing more here (unlock arcade, unlock cars, no
//!   damage, damage timer, 16, arcade timer): other code looks at them;
//! - bits 2 .. 9, 11 and 12 are cleared and each adds one secret car to the list of cars
//!   at `off_75A042`, unless it is there already (the ten functions `sub_470FE9` ..
//!   `sub_471332`, plain functions here). Bit 6, "Max Power", also adds its car to the
//!   list at `off_759D16`, but only when it was not in the first. The other nine have
//!   the same code for the second list, which is never reached;
//! - bit 10 is cleared and does nothing;
//! - bits 17 and 18 stay set and store 1 in `dword_75ACCC` (the A8 game; the reversed
//!   tracks).
//!
//! Callers: `system::load_game_data` with the bits 0x1FBC (when the setting `allsecretcars` is 1)
//! and `sub_480CBF`, which finds a cheat's name typed as the fourth player's name. Refuses a list
//! without its end, or without room for a car more.

use crate::errors::BadInput;
use crate::menu::owner::menus;
use crate::menu::remove_secret_cars::{FIRST, List, SECOND, read_list, write_list};
use rc2k_core::menu::car_lists;

// ---- game data ----

/// By bit (what each does: `rc2k_core::menu::car_lists::CHEATS`): the log line's text and
/// the cheat's car (an index into `CarTable.descriptions`), if any.
const CHEATS: [(&std::ffi::CStr, Option<usize>); 19] = [
    (c"Cheat 1 (Unlock Arcade)", None),
    (c"Cheat 2 (Unlock Cars)", None),
    (c"Cheat 3 (Spud)", Some(27)),
    (c"Cheat 4 (Sheep)", Some(29)),
    (c"Cheat 5 (Cow)", Some(28)),
    (c"Cheat 6 (MF Sports)", Some(23)),
    (c"Cheat 7 (Max Power)", Some(6)),
    (c"Cheat 8 (Local Shop)", Some(32)),
    (c"Cheat 9 (Bolt's Pug)", Some(25)),
    (c"Cheat 10 (Skip)", Some(26)),
    (c"Cheat 11", None),
    (c"Cheat 12 (MF Hatchback)", Some(24)),
    (c"Cheat 13 (Sheep Wagon)", Some(31)),
    (c"Cheat 14 (No Damage)", None),
    (c"Cheat 15 (Damage Timer)", None),
    (c"Cheat 16", None),
    (c"Cheat 17 (Arcade Timer)", None),
    (c"Cheat 18 (Enable A8 Game)", None),
    (c"Cheat 19 (Reverse Sim Tracks)", None),
];

// ---- Rust version ----

/// `log` gets each line's text.
pub fn apply_cheats(log: &mut dyn FnMut(&[u8])) -> Result<(), BadInput> {
    let play = &menus().play;
    // Converted: `rc2k_core::menu::car_lists::apply_cheats`.
    {
        let cheats: u32 = play.cheats.cheats.get();
        let list = |l: List| -> Result<car_lists::List, BadInput> {
            Ok(car_lists::List {
                cars: read_list(l)?,
                room: l.1,
                changed: false,
            })
        };
        let (mut first, mut second) = (list(FIRST)?, list(SECOND)?);
        let a = car_lists::apply_cheats(cheats, &mut first, &mut second, |bit| {
            CHEATS[bit].1.map_or(0, crate::menu::cars_store::car_handle)
        })?;
        // Nothing has been written or logged up to here.
        play.cheats.cheats.set(a.cheats);
        if first.changed {
            write_list(FIRST, &first.cars);
        }
        if second.changed {
            write_list(SECOND, &second.cars);
        }
        if a.stores {
            crate::race::owner::race()
                .shared
                .state
                .settings
                .a8_game
                .set(1u32);
        }
        for bit in a.logged {
            log(CHEATS[bit].0.to_bytes());
        }
        Ok(())
    }
}

// logfile::log_write_l1_nl
