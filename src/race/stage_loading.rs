//! `race::load_stage` (0x444040): "SHAUNINITEVERYTHING": the loading
//! picture (unless `in_credits`), the cars' sounds woken, `race::loadlevel_init`, the
//! in-game music, the controls, race init (`drive::race_init`, the game's until ported), the
//! clock restarted and the race's starting values; each step logged and indented.

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

/// `race::load_stage` (0x444040): a line break, a rule, "START", "loading screen" (indent in): the
/// loading picture (`menu::show_loading_picture`) unless `in_credits`, `last_kept` -1,
/// `race::wake_car_sounds`; then each with its word and the indent round it: `loadlevel_init`, the
/// music (`game_music` with `Finish.no_place`, else `game_music_8`), `create_controls`, race init
/// (`drive::race_init`) and `scene::restart_clock`; the starting values (`STARTS` with the clock,
/// the blocks' +0xFB4 3 and +0xFB0 8, `LAST_STARTS`); "FINIS", a rule, a line break.
#[cfg(debug_ui)]
pub fn load_stage(race: &Race) -> Result<(), Refused> {
    load_stage_in(race, crate::owners::boundary())
}

/// `load_stage` with the owners given.
pub fn load_stage_in(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (system, gfx, input, scene) = (o.system, o.gfx, o.input, o.scene);
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
    if !race.shared.work.in_credits.is_set()
    /* dword_71C3E0 */
    {
        // As `menu::show_loading_picture`.
        crate::gfx::show_picture_file::show_picture_file(
            &o.menus.tables.texts.loading_pcx,
            gfx,
            system,
        )?;
    }
    race.replay.replay_frames.last_kept.set(u32::MAX);
    crate::race::sounds::wake_car_sounds(race, o.sound)?;
    indent(-3);
    log();
    indent(3);
    use crate::race::loading_bar_frame::STAGE_LOADING;
    STAGE_LOADING.store(true, std::sync::atomic::Ordering::Relaxed);
    crate::race::heard_turn::forget();
    #[cfg(feature = "remote")]
    crate::remote::race_loading(o.effects);
    let begun = std::time::Instant::now();
    crate::race::loadlevel::loadlevel_init(race, o)?;
    STAGE_LOADING.store(false, std::sync::atomic::Ordering::Relaxed);
    let (shown, micros) = crate::race::loading_bar_frame::shown();
    crate::framework::log(&format!(
        "race::load_stage: loaded in {} ms, {shown} bar frames in {} ms",
        begun.elapsed().as_millis(),
        micros / 1000
    ));
    indent(-3);
    log();
    indent(3);
    if race.replay.finish.no_place.get() != 0 {
        crate::race::music::game_music(&race.shared);
    } else {
        crate::race::music::game_music_8();
    }
    indent(-3);
    log();
    indent(3);
    crate::race::create_controls::create_controls(input);
    indent(-3);
    log();
    indent(3);
    crate::drive::race_init::race_init(race, system, o.sound, scene, o.effects, o.net)?;
    crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(),
    );
    let clock: u32 = race.shared.physics_tables.scene_clock.get();
    race.frame.clock_ticks.clock_b.set(clock);
    race.frame.clock_ticks.sound_ticks.set(0);
    race.shared.physics_tables.race_clock_now.set(clock);
    race.shared.physics_tables.race_clock_base.set(clock);
    race.frame.panel_and_overlay.overlay.start_lights_lit.set(0);
    race.frame.panel_and_overlay.panel.lights[0]
        .lit
        .set(u32::MAX);
    race.frame.panel_and_overlay.panel.lights[1]
        .lit
        .set(u32::MAX);
    for car in &race.car_blocks().cars[..8] {
        car.light_hold_steps.set(3);
    }
    for car in &race.car_blocks().cars[..8] {
        car.value_jump_steps.set(8);
    }
    race.shared.switches.scene_set_up.set(1);
    race.shared.switches.view_marks_on.set(1);
    race.physics.ground.stage_loading.set(0);
    race.shared.view.view_on[0].set(1);
    race.shared.view.view_on[1].set(1);
    indent(-3);
    log();
    log();
    newline();
    Ok(())
}
