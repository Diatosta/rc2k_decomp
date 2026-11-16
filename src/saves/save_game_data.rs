//! Writes what the game keeps between runs: `saves::save_options`, then
//! `saves::save_best_times`. Called by the main loop after the front end is left, and
//! `menu::arcade_level_end`.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::saves::owner::Saves;

/// The two parts, each saying whether it wrote its file. Returns what the second said.
pub fn save_game_data(
    menus: &Menus,
    save_options: &mut dyn FnMut() -> Result<bool, Refused>,
    save_best_times: &mut dyn FnMut() -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    // The front end fills these as it ends; closing the window before that saves too, and
    // lists never filled in the file would refuse every later start.
    crate::saves::list_car_numbers::list_car_numbers_in(menus)?;
    save_options()?;
    save_best_times()
}

/// The two parts on the game's files: whether the best times were written (neither caller,
/// `system::main_loop` and `menu::arcade_level_end`, reads it).
pub fn save_in_game(o: Owners<'static>, saves: &Saves) -> Result<bool, Refused> {
    save_game_data(
        o.menus,
        &mut || crate::saves::options_files::save(o),
        &mut || crate::saves::save_best_times::in_game(saves),
    )
}
