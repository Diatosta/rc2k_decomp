//! `race::play_movie_loop`: the viewer's replay played as a race: the skid marks set up
//! (`race::init_skidmarks`), "dpg replay", the block played from (`replay_block`) and the one it
//! stops at (`play_end`) logged; the race's state 2 (`replay_mode`), the clock restarted
//! (`scene::restart_clock`, `scene_clock` into `race_clock_base`); then each frame the input
//! (`input::update`), a key pressed (`keys_pressed + 1`) ending it, the race's step
//! (`drive::race_step`) and frame (`drive::race_frame`), until the block played is the one to stop
//! at; "dpg replay exit".

use crate::errors::Refused;
use crate::race::owner::Race;

const TEXTS: [&std::ffi::CStr; 4] = [
    c"dpg replay",
    c"[replaycurchunk]",
    c"[replayplaystop]",
    c"dpg replay exit",
];

fn log_text(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(&text, newline, 1);
}

fn log_number(n: u32) {
    crate::logfile::log_int::log_int(n as i32, true, 1);
}

pub(super) fn play_movie_loop(
    race: &Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let input = o.input;
    use crate::drive::race_parts;
    crate::race::stage_prefs::init_skidmarks(race)?;
    log_text(TEXTS[0], false);
    log_text(TEXTS[1], false);
    log_number(race.replay.replay_view().replay_block.get());
    log_text(TEXTS[2], false);
    log_number(race.replay.replay_view().play_end.get());
    race.course.number_vars.get().replay_mode.set(2);
    // A replay rendered to a video runs on the video's clock from here on.
    crate::video::race_loop_begins();
    race_parts::restart_clock(race, &o.system.session.picture_clock);
    let scene_clock = race.shared.run().scene_clock;
    race.shared.run_mut().race_clock_base = scene_clock;
    let pad_at_start = input.pad_marker();
    let _racing = crate::race::blend::Racing::begin(race);
    loop {
        race_parts::input_update(o);
        if input.devices.keys_pressed.get(1) != 0 || input.pad_pressed_since(pad_at_start) {
            break;
        }
        { crate::drive::race_step::race_step(o, race) }?;
        race_parts::race_frame(race, o)?;
        if race.replay.replay_view().replay_block.get() == race.replay.replay_view().play_end.get()
        {
            break;
        }
    }
    log_text(TEXTS[3], false);
    Ok(())
}
