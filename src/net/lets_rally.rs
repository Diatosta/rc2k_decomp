//! `net::lets_rally` (0x4170E0): "NET: Lets rally", one network race: racing
//! (`dword_52AD84` 1), `dword_75ACE0` 1, `dword_75ACD8` 0, this player's car and colours to
//! `dword_75ACE8` and `dword_75AE2C`; the first player that is not the host found
//! (`dword_52A924`; none: carry); the music stopped, the race's screen mode, the loading
//! picture, the race's sounds (`race::set_up_race_sound_objects`,
//! `sound::set_up_race_sounds`), the race set up (`net::set_up_race`); the clocks noted, the
//! sync table to clear, the players listed; then empty frames (`screens::run_frame`, eax 0)
//! with a key each (`input::take_key`, ebx 1) until Escape, or until everyone waits
//! (`net::race_wait`): then the front surface cleared, the messages taken
//! (`net::copy_received`) and the race driven (`net::lets_race`); after it, everything
//! tidied (`race::tidy_everything`), the music stopped, the menus' screen mode and loading
//! picture, the race's sounds cleared, the sound settings, the engine and the front end's
//! scene set up again (inside `pusha` .. `popa`); with ebx not 0 (Escape, or the car not
//! finished) carry, else the race's time recorded (`net::race_over`).
//!
//! In:  nothing
//! Out: whether the original sets the carry (as above); every register kept (`pusha` ..
//!      `popa`)
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{log, run_frame};
use crate::net::owner::Net;
use crate::race::owner::Race;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use rc2k_core::net::record;

/// The Rust version: true when the original ends with the carry set (no player but the
/// host, Escape, or the car not finished: the link ends).
pub fn lets_rally(race: &Race, net: &Net) -> Result<bool, Refused> {
    use crate::race::sound_tables;
    log(VIEWER_PANEL.text_ptr(texts::NET_LETS_RALLY), true, 2);
    net.vars.racing.set(1);
    let own = &net.vars.player_block(net.vars.player.get()).record;
    race.shared.state.settings.race_mode.set(1);
    race.shared.state.settings.run_kind.set(0);
    // Converted: `rc2k_core::net::record` (`CAR`, `COLOURS`, `first_not_host`).
    race.shared
        .state
        .settings
        .car_kind_current
        .set(own.head.get(record::CAR) as u32);
    race.shared.state.settings.cars[0]
        .gearing
        .set(own.head.get(record::COLOURS) as u32);
    let state = |n: u32| net.vars.player_block(n).record.state();
    let Some(first) = record::first_not_host(net.vars.players.get(), state) else {
        return Ok(true);
    };
    net.vars.first_host.set(first);
    crate::sound::stop_music::stop_music();
    crate::race::race_shell::set_race_screen_mode();
    crate::menu::show_loading_picture::show_loading_picture(crate::menu::owner::menus())?;
    sound_tables::set_up_race_sound_objects(race, &sound_tables::ObjectVars::game())?;
    crate::sound::set_up_race_sounds::set_up_race_sounds()?;
    crate::net::set_up_race::set_up_race(race, &net.vars)?;
    let tick = crate::system::tick_count::tick_count();
    net.vars.clock_a.set(tick);
    net.vars.sync_clock.set(tick);
    net.vars.sync_first.set(1);
    crate::net::list_players::list_players_regs(&net.vars)?;
    // The race's time and whether the car did not finish (the original's eax and ebx);
    // Escape leaves ebx 1.
    let (time, not_finished) = loop {
        run_frame(|| Ok::<(), Refused>(()))?;
        if crate::input::take_key::take_key() == 1 {
            break (1, 1);
        }
        if crate::net::race_wait::race_wait(net)? != 0 {
            crate::gfx::clear_front_surface::clear_front_surface()?;
            crate::net::copy_received::copy_received(&net.vars)?;
            break crate::net::lets_race::lets_race(race, &net.vars)?;
        }
    };
    crate::race::race_end::tidy_everything(race)?;
    crate::sound::stop_music::stop_music();
    crate::platform::set_menu_mode_now();
    crate::menu::show_loading_picture::show_loading_picture(crate::menu::owner::menus())?;
    sound_tables::clear_race_sound_objects()?;
    crate::sound::apply_settings::apply_settings()?;
    crate::scene::set_up_engine::set_up_engine()?;
    crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
    crate::scene::set_up_engine::set_up_engine()?;
    if not_finished != 0 {
        return Ok(true);
    }
    crate::net::race_over::race_over(net, time)?;
    Ok(false)
}
