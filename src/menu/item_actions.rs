//! Item actions of menu items: `menu::show_intro` (films, then back), `previous_save_page` (the
//! page one less, from 0 round to 9), `service::open_steering_box` / `open_brake_box` (the box
//! turned over, its page 3 or 4), `view_stage_details` (a time trial's stage details, else the
//! stage view), `new_simulation_if_unlocked` (the simulation, else the refusal's sound 8 and a
//! quarter of a second's sleep) and `start_arcade` (level and stage 0, the first player's units
//! taken, the arcade event).

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::owners::Owners;

/// `menu::show_intro`: the films (idle mode), then back.
pub fn show_intro(o: Owners<'static>) -> Result<(), Refused> {
    crate::menu::play_films::play_films(o, true)?;
    crate::menu::go_back::go_back(o.menus);
    Ok(())
}

pub fn previous_save_page(tables: &Tables) {
    tables
        .pages
        .save_page
        .set(rc2k_core::menu::steps::savegame_page(
            tables.pages.save_page.get(),
            false,
        ));
}

/// The steering (page 3) or brake (page 4) box.
pub fn open_service_box(screens: &Screens, page: u32) {
    screens
        .state
        .service_ui
        .box_shown
        .set(!screens.state.service_ui.box_shown.is_set());
    screens.state.service_ui.box_page.set(page);
}

/// `menu::view_stage_details`: the stage information screen.
pub fn view_stage_details(screens: &Screens, entry: &Entry) -> Result<(), Refused> {
    let time_trial =
        rc2k_core::menu::stage_screens::time_trial_details(screens.event.event_type.get());
    if time_trial {
        run_screen(ScreenId::TimeTrialStageDetails, entry)?;
    } else {
        run_screen(ScreenId::ViewStage, entry)?;
    }
    Ok(())
}

/// `menu::new_simulation_if_unlocked`.
pub fn new_simulation_if_unlocked(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    use rc2k_core::menu::stage_screens::{REFUSED_SLEEP, REFUSED_SOUND, simulation_unlocked};
    if simulation_unlocked(o.race.shared.state.settings.a8_game.get()) {
        // (An item of the typed tables: it runs with no carried values, as the menus'
        // items do.)
        crate::menu::new_simulation_loop::run(o)?;
    } else {
        crate::sound::play_menu_sound::play_menu_sound_in_game(
            o.sound,
            o.audio,
            o.menus,
            o.system,
            REFUSED_SOUND,
        );
        crate::system::sleep::sleep(REFUSED_SLEEP);
    }
    Ok(())
}

/// `menu::start_arcade`: the arcade's level and stage 0, the first player's units taken
/// and the arcade event run.
pub fn start_arcade(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (menus, play) = (o.menus, &o.menus.play);
    play.arcade.level.set(0);
    play.arcade.stage.set(0);
    crate::menu::take_player1_units::take_player1_units(menus, o.strings)?;
    crate::menu::arcade_event::arcade_event(o)
}
