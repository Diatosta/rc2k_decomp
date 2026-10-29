//! `menu::arcade_level_end` (0x48AD73): an arcade level's end. Unless the level was won
//! (`dword_758768` 1): tune 7 (`sound::play_tune_7`), `menu::screen_488f4c_loop` and
//! `dword_7587F4` 1. Won: tune 6, `menu::arcade_finale`; the first time the level is won
//! (`byte_758800[level]` 0, made 1) its secret car: esi `off_75A12E`,
//! `menu::car_specs_47af4c_loop`, `add_shown_car`, `remove_shown_car`; then
//! `saves::save_game_data`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! The callees get eax the level (from the won level's test on) and esi as above. Refuses (before
//! anything) a level past 5.

use crate::errors::Refused;
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::results_store::RESULT_FIGURES;
use crate::menu::screen_loop::{Carried, run_screen_carried};

/// The Rust version. `c` the carried values the level end is entered with; the original
/// keeps every register of the caller (`pusha` .. `popa`).
pub fn arcade_level_end(c: &Carried) -> Result<(), Refused> {
    let mut r = *c;
    // Converted: `rc2k_core::menu::arcade` (`won`, `won_level`, `first_win`).
    use rc2k_core::menu::arcade::{first_win, won, won_level};
    if !won(RESULT_FIGURES.positions[0].get()) {
        crate::sound::play_tune_7::play_tune_7();
        run_screen_carried(&crate::menu::screen_488f4c_loop::SCREEN, &mut r)?;
        ARCADE_GAME.retry.set(1u32);
        return Ok(());
    }
    let level = won_level(ARCADE_GAME.level.get())?;
    crate::sound::play_tune_6::play_tune_6();
    crate::menu::arcade_finale::arcade_finale()?;
    r.arg = level;
    if first_win(ARCADE_GAME.levels_won.get(level as usize)) {
        ARCADE_GAME.levels_won.set(level as usize, 1);
        r.cars = MENU_LISTS.shown_cars[0].addr();
        run_screen_carried(&crate::menu::car_specs_47af4c_loop::SCREEN, &mut r)?;
        crate::menu::arcade_steps::add_shown_car()?;
        crate::menu::arcade_steps::remove_shown_car(&mut || {
            crate::saves::list_car_numbers::list_car_numbers().map_err(Refused::from)
        })?;
    }
    crate::saves::save_game_data::save_in_game()?;
    Ok(())
}
