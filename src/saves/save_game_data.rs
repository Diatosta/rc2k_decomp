//! Writes what the game keeps between runs: `saves::save_options`, then `saves::save_best_times`.

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

/// The options alone (the options files), written when a page that changes them is left. Not
/// before the game's data is loaded (a save then would write blanks); a failure is logged and
/// the game goes on, the exit save tries again.
pub fn save_settings_in_game(o: Owners<'static>) {
    // After "delete all saved data" nothing is written back.
    if crate::saves::delete_all::deleted() {
        return;
    }
    let tables = &o.saves.best_times;
    if [tables.allocated.get(), tables.loaded.get()] != [1, 1] {
        return;
    }
    let done = crate::saves::list_car_numbers::list_car_numbers_in(o.menus)
        .map_err(Refused::from)
        .and_then(|()| crate::saves::options_files::save(o));
    if let Err(e) = done {
        crate::logfile::program_log::log(&format!(
            "saves: the settings were not saved on leaving a page: {e:?}"
        ));
    }
}
