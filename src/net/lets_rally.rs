//! One network race ("NET: Lets rally"): sets the race up, then runs empty frames until Escape
//! or until everyone is waiting (`net::race_wait`); then the messages are taken and the race
//! driven (`net::lets_race`); afterwards everything is tidied and the menus' state restored. If
//! the race was not left (Escape, or the car not finishing), its time is recorded
//! (`net::race_over`).

use crate::errors::Refused;
use crate::net::lobby::{log_bytes, run_frame};
use crate::owners::Owners;
use crate::race::viewer_screen_store::texts;
use rc2k_core::net::record;

/// True when the link ends: no player but the host, Escape, or the car not finished.
pub fn lets_rally(o: Owners<'static>) -> Result<bool, Refused> {
    let (race, net, sound) = (o.race, o.net, o.sound);
    use crate::race::sound_tables;
    log_bytes(
        &crate::race::viewer_screen_store::NET_TEXTS.text(texts::NET_LETS_RALLY),
        true,
        2,
    );
    net.lobby().racing.set(1);
    let own = &net.vars.player_block(net.race_slot.player())?.record;
    race.shared.state.settings.race_mode.set(1);
    race.shared.state.settings.run_kind.set(0);
    // The logic is in `rc2k_core::net::record` (`CAR`, `COLOURS`, `first_not_host`).
    race.shared
        .state
        .settings
        .car_kind_current
        .set(own.head.get(record::CAR) as u32);
    race.shared.state.settings.cars[0]
        .controls
        .gearing
        .set(own.head.get(record::COLOURS) as u32);
    let state = |n: u32| net.vars.player_block_or_end(n).record.state();
    let Some(first) = record::first_not_host(net.race_slot.players(), state) else {
        return Ok(true);
    };
    net.lobby().first_host.set(first);
    // The host's computer drivers, known before the sound objects are made for the cars.
    net.race_slot.set_bots(crate::net::bots::listed_count(net));
    crate::sound::stop_music::stop_music(o);
    crate::race::race_shell::set_race_screen_mode(o.gfx, o.system);
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    sound_tables::set_up_race_sound_objects(o, &sound_tables::ObjectVars::of(sound))?;
    crate::sound::set_up_race_sounds::set_up_race_sounds(o)?;
    crate::net::set_up_race::set_up_race(race, &net.vars, o)?;
    let tick = crate::clock::tick_count::tick_count(&o.system.session.picture_clock);
    net.lobby().listed_at.set(tick);
    net.lobby().sync_clock.set(tick);
    net.vars.sync_first.set(1);
    crate::net::list_players::list_players_regs(net)?;
    // The race's time and whether the car did not finish; Escape leaves the second 1.
    // The pad's Back sends Escape; Start (the race's pause button) leaves too.
    let pad_at_start = o.input.pad_marker();
    let (time, not_finished) = loop {
        run_frame(o, || Ok::<(), Refused>(()))?;
        if crate::input::take_key::take_key(o.input) == 1 || o.input.pad_start_since(pad_at_start) {
            break (1, 1);
        }
        if crate::net::race_wait::race_wait(net, &o.system.session.picture_clock)? != 0 {
            crate::gfx::clear_front_surface::clear_front_surface(o.gfx, o.system)?;
            crate::net::copy_received::copy_received(net, &mut net.lock_received())?;
            break crate::net::lets_race::lets_race(o)?;
        }
    };
    crate::race::race_end::tidy_everything(race, o)?;
    crate::sound::stop_music::stop_music(o);
    crate::platform::set_menu_mode_now(o.gfx, o.system);
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    sound_tables::clear_race_sound_objects(o)?;
    crate::sound::apply_settings::apply_settings(o)?;
    crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?;
    crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
    crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?;
    if not_finished != 0 {
        return Ok(true);
    }
    net.lobby()
        .replay_kept
        .set(u8::from(race.shared.replay_file().replay_closed.get() == 1));
    crate::net::race_over::race_over(net, time)?;
    Ok(false)
}
