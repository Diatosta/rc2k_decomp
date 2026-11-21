//! `race::load_stage`: "SHAUNINITEVERYTHING": shows the loading picture (unless `in_credits`),
//! wakes the cars' sounds, runs `race::loadlevel_init`, sets up the in-game music and the
//! controls, runs race init (`drive::race_init`), restarts the clock and sets the race's starting
//! values; each step is logged and indented.

use crate::errors::Refused;
use crate::race::owner::Race;

/// "------", "SHAUNINITEVERYTHING - START", ": loading screen / ..", ": loadlevel_init",
/// ": doingamemusic", ": createcontrols", ": race init", "- FINIS", "------".
const WORDS: [&std::ffi::CStr; 9] = [
    c"----------------------------",
    c"SHAUNINITEVERYTHING - START",
    c"SHAUNINITEVERYTHING : loading screen / sounds",
    c"SHAUNINITEVERYTHING : loadlevel_init",
    c"SHAUNINITEVERYTHING : doingamemusic",
    c"SHAUNINITEVERYTHING : createcontrols",
    c"SHAUNINITEVERYTHING : race init",
    c"SHAUNINITEVERYTHING - FINIS",
    c"----------------------------",
];

/// `race::load_stage`: logs a line break, a rule, "START" and "loading screen" (indenting). Shows
/// the loading picture (`menu::show_loading_picture`) unless `in_credits`, sets `last_kept` to -1
/// and wakes the car sounds (`race::wake_car_sounds`). Then, each with its word and the indent
/// round it: `loadlevel_init`, the music (`game_music` with `Finish.no_place`, else
/// `game_music_8`), `create_controls`, race init (`drive::race_init`) and `scene::restart_clock`.
/// Last come the starting values (the clocks, the start lights, and 3 light hold steps for each
/// of the eight cars), "FINIS", a rule and a line break.
pub fn load_stage(race: &'static Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let input = o.input;
    crate::remote::reset_texture_hash();
    crate::logfile::load_times::begin("load_stage: loading picture, car sounds");
    let indent = |by: i32| {
        crate::logfile::write_line::state::INDENT
            .atomic()
            .fetch_add(by as u32, std::sync::atomic::Ordering::Relaxed);
    };
    let mut word = 0;
    let mut log = || {
        crate::logfile::log_write::log_nul(&WORDS[word], true, 1);
        word += 1;
    };
    let newline = || crate::logfile::log_write::log_nul(&c"", true, 1);
    newline();
    log();
    log();
    log();
    indent(3);
    crate::logfile::load_times::mark("show loading picture");
    if !race.shared.work.get().in_credits.is_set() {
        // The same picture as `menu::show_loading_picture` shows.
        crate::gfx::show_picture_file::show_picture_file(o, &o.menus.texts.loading_pcx)?;
        crate::race::loading_bar_frame::picture_shown();
    }
    race.shared.view.get().lights_follow_view.set(0u32);
    race.replay.replay_frames_mut().last_kept = u32::MAX;
    crate::logfile::load_times::mark("wake car sounds");
    crate::race::sounds::wake_car_sounds(race, o.sound, o.audio, &o.system.session.picture_clock)?;
    indent(-3);
    log();
    indent(3);
    use crate::race::loading_bar_frame::STAGE_LOADING;
    STAGE_LOADING.store(true, std::sync::atomic::Ordering::Relaxed);
    o.race.shared.heard_turn.get().forget();
    crate::menu::launch::stage_loading(o.race, o.race.effects.get());
    crate::remote::race_loading(o.race, o.race.effects.get());
    let begun = std::time::Instant::now();
    crate::race::loadlevel::loadlevel_init(
        race,
        o,
        &mut crate::race::loading_bar_frame::LoadingBar::new(o),
    )?;
    crate::remote::course_hashes(race, o.scene, "load");
    STAGE_LOADING.store(false, std::sync::atomic::Ordering::Relaxed);
    let (shown, micros) = crate::race::loading_bar_frame::shown();
    crate::program_log::log(&format!(
        "race::load_stage: loaded in {} ms, {shown} bar frames in {} ms",
        begun.elapsed().as_millis(),
        micros / 1000
    ));
    indent(-3);
    log();
    indent(3);
    crate::logfile::load_times::phase("load_stage: music");
    if race.replay.no_place() != 0 {
        crate::race::music::game_music(&race.shared, o);
    } else {
        crate::race::music::game_music_8(o);
    }
    indent(-3);
    log();
    indent(3);
    crate::logfile::load_times::phase("load_stage: create controls");
    crate::race::create_controls::create_controls(input, race.shared.view.get().players.get());
    indent(-3);
    log();
    indent(3);
    crate::logfile::load_times::phase("load_stage: race init");
    crate::drive::race_init::race_init(o, race)?;
    crate::logfile::load_times::phase("load_stage: starting values");
    crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::clock::tick_count::tick_count(&o.system.session.picture_clock),
    );
    let clock: u32 = race.shared.run().scene_clock;
    race.shared.run_mut().race_clock_now = clock;
    race.shared.run_mut().race_clock_base = clock;
    race.frame.overlay.get().start_lights_lit.set(0);
    race.frame.panel.borrow_mut().lights[0].lit = u32::MAX;
    race.frame.panel.borrow_mut().lights[1].lit = u32::MAX;
    for car in &race.car_blocks().cars[..8] {
        car.light_hold_steps.set(3);
    }
    for car in &race.car_blocks().cars[..8] {
        car.value_jump_steps.set(8);
    }
    race.shared.switches.borrow_mut().scene_set_up = 1;
    race.shared.switches.borrow_mut().view_marks_on = 1;
    race.shared.view.get().view_on[0].set(1);
    race.shared.view.get().view_on[1].set(1);
    indent(-3);
    log();
    log();
    newline();
    crate::remote::started_hashes(race);
    crate::logfile::load_times::end();
    Ok(())
}
