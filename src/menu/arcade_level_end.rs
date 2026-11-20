//! An arcade level's end. A level not won (the first player's place is not 1) plays the losing
//! tune, shows the level-end results and sets the retry flag. A won level plays the winning tune
//! and the finale; the first time it is won its secret car is shown and added to the cars, and
//! the game is saved.
//!
//! Refused before anything: a level past 5.

use crate::errors::Refused;
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};

pub fn arcade_level_end(
    menus: &Menus,
    _sound: &crate::sound::owner::Sound,
    entry: &Entry,
) -> Result<(), Refused> {
    let play = &menus.play;
    use rc2k_core::menu::arcade::{first_win, won, won_level};
    if !won(play.results.players[0].position.get()) {
        crate::sound::play_tune_7::play_tune_7(entry.owners);
        run_screen(ScreenId::ArcadeLevelEndResults, entry)?;
        play.arcade.write().retry = 1u32;
        return Ok(());
    }
    let level = won_level(play.arcade.read().level)?;
    crate::sound::play_tune_6::play_tune_6(entry.owners);
    crate::menu::arcade_finale::arcade_finale(entry.owners)?;
    if first_win(play.arcade.opened(level as usize)) {
        play.arcade.open(level as usize);
        let shown = entry.with_list(Some(CarList::Shown));
        run_screen(ScreenId::CarSpecsDisplay, &shown)?;
        crate::menu::arcade_steps::add_shown_car(menus)?;
        crate::menu::arcade_steps::remove_shown_car(menus, &mut || {
            crate::saves::list_car_numbers::list_car_numbers_in(menus).map_err(Refused::from)
        })?;
    }
    crate::saves::save_game_data::save_in_game(entry.owners, entry.owners.saves)?;
    Ok(())
}
