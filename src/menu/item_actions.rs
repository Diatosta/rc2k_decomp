//! Item actions the standalone met unported (2026-10-19, the owner's run without RAL.EXE):
//! - `menu::show_intro` (0x47058E): the options' "intro" item: the films played
//!   (`menu::play_films`, eax 1), then back (`menu::go_back`);
//! - `menu::previous_save_page` (0x46D3F2): the championship load page's left arrow: the
//!   page (`dword_75523C`) one less, from 0 round to 9;
//! - `service::open_steering_box`, `service::open_brake_box` (0x46B706, 0x46B71A): the
//!   service area's steering sensitivity and brake bias items: the box switch
//!   (`dword_75AA9C`) turned over and its page (`dword_75AAA0`) 3, or 4;
//! - `menu::view_stage_details` (0x47037E): "view stage info": for a time trial
//!   (`g_eventType` 3) `menu::time_trial_stage_details_loop`, else `menu::view_stage_loop`;
//! - `menu::new_simulation_if_unlocked` (0x4A96BF): with `dword_75ACCC` 1
//!   `menu::new_simulation_loop`, else the refusal's sound (`sound::play_menu_sound` 8) and a
//!   quarter of a second's sleep;
//! - `menu::start_arcade` (0x487CAA): the arcade's level and stage (`g_arcadeLevel`,
//!   `g_arcadeStage`) 0, the first player's units taken (`menu::take_player1_units`) and the
//!   arcade event run (`menu::arcade_event`).
//!
//! In:  nothing (eax, the item's dword, is not read but by `show_intro`, which sets it)
//! Out: the pure ones and those in `pusha` / `popa` every register kept; the others the
//! registers their last call leaves

use crate::errors::Refused;
use crate::menu::owner::Play;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::menu::screen_loop::{Entry, run_screen};

// ---- the Rust versions ----

/// `menu::show_intro`: the films (idle mode), then back.
pub fn show_intro() -> Result<(), Refused> {
    crate::menu::play_films::play_films(true)?;
    crate::menu::go_back::go_back();
    Ok(())
}

pub fn previous_save_page(tables: &Tables) {
    // Converted: `rc2k_core::menu::steps::savegame_page`.
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
        .set(screens.state.service_ui.box_shown.get() ^ 1);
    screens.state.service_ui.box_page.set(page);
}

/// `menu::view_stage_details`: the stage information screen.
pub fn view_stage_details(screens: &Screens) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::stage_screens::time_trial_details`.
    let time_trial =
        rc2k_core::menu::stage_screens::time_trial_details(screens.event.event_type.get());
    if time_trial {
        run_screen(
            &crate::menu::time_trial_stage_details_loop::SCREEN,
            &Entry::default(),
        )?;
    } else {
        run_screen(&crate::menu::view_stage_loop::SCREEN, &Entry::default())?;
    }
    Ok(())
}

/// `menu::new_simulation_if_unlocked` (it reads no register).
pub fn new_simulation_if_unlocked() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::stage_screens::simulation_unlocked`.
    use rc2k_core::menu::stage_screens::{REFUSED_SLEEP, REFUSED_SOUND, simulation_unlocked};
    if simulation_unlocked(
        crate::race::owner::race()
            .shared
            .state
            .settings
            .a8_game
            .get(),
    ) {
        // (An item of the typed tables: it runs with no carried values, as the menus'
        // items do.)
        crate::menu::new_simulation_loop::run()?;
    } else {
        crate::sound::play_menu_sound::play_menu_sound_in_game(REFUSED_SOUND);
        crate::system::sleep::sleep(REFUSED_SLEEP);
    }
    Ok(())
}

/// `menu::start_arcade`: the arcade's level and stage 0, the first player's units taken
/// and the arcade event run.
pub fn start_arcade(play: &Play) -> Result<(), Refused> {
    play.arcade.level.set(0);
    play.arcade.stage.set(0);
    crate::menu::take_player1_units::take_player1_units()?;
    crate::menu::arcade_event::arcade_event()
}
