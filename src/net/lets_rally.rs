//! `net::lets_rally` (0x4170E0): "NET: Lets rally", one network race: racing
//! (`racing` 1), the race mode 1, the settings' second value 0, this player's car and
//! colours to the settings' first car; the first player that is not the host found
//! (`first_host`; none: carry); the music stopped, the race's screen mode, the loading
//! picture, the race's sounds (`race::set_up_race_sound_objects`,
//! `sound::set_up_race_sounds`), the race set up (`net::set_up_race`); the clocks noted, the
//! sync table to clear, the players listed; then empty frames (`screens::run_frame`)
//! with a key each (`input::take_key`) until Escape, or until everyone waits
//! (`net::race_wait`): then the front surface cleared, the messages taken
//! (`net::copy_received`) and the race driven (`net::lets_race`); after it, everything
//! tidied (`race::tidy_everything`), the music stopped, the menus' screen mode and loading
//! picture, the race's sounds cleared, the sound settings, the engine and the front end's
//! scene set up again; unless the race was left (Escape, or the car not
//! finished), the race's time recorded (`net::race_over`).
//!
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{log_bytes, run_frame};
use crate::owners::Owners;
use crate::race::viewer_screen_store::texts;
use rc2k_core::net::record;

/// True when the original ends with the carry set (no player but the
/// host, Escape, or the car not finished: the link ends).
pub fn lets_rally(o: Owners<'static>) -> Result<bool, Refused> {
    let (race, net, sound) = (o.race, o.net, o.sound);
    use crate::race::sound_tables;
    log_bytes(&race.viewer_panel().text(texts::NET_LETS_RALLY), true, 2);
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
    crate::menu::show_loading_picture::show_loading_picture(o.menus)?;
    sound_tables::set_up_race_sound_objects(race, &sound_tables::ObjectVars::game(), sound)?;
    crate::sound::set_up_race_sounds::set_up_race_sounds(sound)?;
    crate::net::set_up_race::set_up_race(race, &net.vars, o)?;
    let tick = crate::system::tick_count::tick_count();
    net.edit().clock_a.set(tick);
    net.vars.sync_clock.set(tick);
    net.vars.sync_first.set(1);
    crate::net::list_players::list_players_regs(&net.vars)?;
    // The race's time and whether the car did not finish (the original's result registers);
    // Escape leaves the second 1.
    let (time, not_finished) = loop {
        run_frame(|| Ok::<(), Refused>(()))?;
        if crate::input::take_key::take_key() == 1 {
            break (1, 1);
        }
        if crate::net::race_wait::race_wait(net)? != 0 {
            crate::gfx::clear_front_surface::clear_front_surface()?;
            crate::net::copy_received::copy_received(&net.vars)?;
            break crate::net::lets_race::lets_race(o, &net.vars)?;
        }
    };
    crate::race::race_end::tidy_everything_in(race, o)?;
    crate::sound::stop_music::stop_music();
    crate::platform::set_menu_mode_now();
    crate::menu::show_loading_picture::show_loading_picture(o.menus)?;
    sound_tables::clear_race_sound_objects(sound)?;
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
