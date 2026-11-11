//! `race::play_movie_loop` (0x4607A5): the viewer's replay played as a race: the skid marks set up
//! (`race::init_skidmarks`), "dpg replay", the block played from (`replay_block`) and the one it
//! stops at (`play_end`) logged; the race's state 2 (`replay_mode`), the clock restarted
//! (`scene::restart_clock`, `g_sceneClock` into `race_clock_base`); then each frame the input
//! (`input::update`), a key pressed (`g_keysPressed + 1`) ending it, the race's step
//! (`drive::race_step`) and frame (`drive::race_frame`), until the block played is the one to stop
//! at; "dpg replay exit".
//!
//! Caller: `race::play_movie`.

use crate::errors::Refused;
use crate::race::owner::Race;

const TEXTS: [&std::ffi::CStr; 4] = [
    c"dpg replay",
    c"[replaycurchunk]",
    c"[replayplaystop]",
    c"dpg replay exit",
]; // "dpg replay" ..

fn log_text(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(&text, newline, 1);
}

fn log_number(n: u32) {
    crate::logfile::log_int::log_int(n as i32, true, 1);
}

pub fn play_movie_loop(race: &Race, input: &crate::input::owner::Input) -> Result<(), Refused> {
    use crate::drive::race_parts;
    crate::race::stage_prefs::init_skidmarks(race)?;
    log_text(TEXTS[0], false);
    log_text(TEXTS[1], false);
    log_number(race.replay.replay_view.replay_block.get());
    log_text(TEXTS[2], false);
    log_number(race.replay.replay_view.play_end.get());
    race.course.numbers.replay_mode.set(2); // dword_5C5CE0
    race_parts::restart_clock(race);
    race.shared
        .physics_tables
        .race_clock_base
        .set(race.shared.physics_tables.scene_clock.get());
    loop {
        race_parts::input_update();
        if input.devices.keys_pressed.get(1) != 0 {
            break;
        }
        { crate::drive::race_step::race_step(race, crate::scene::owner::scene()) }?;
        race_parts::race_frame(race)?;
        if race.replay.replay_view.replay_block.get() == race.replay.replay_view.play_end.get() {
            break;
        }
    }
    log_text(TEXTS[3], false);
    Ok(())
}
