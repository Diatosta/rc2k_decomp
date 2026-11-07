//! `race::credits_race` (0x463666): the credits' race (the demonstration the films play
//! between them): `dword_5C5CE0` 0, `g_holdCarRevs` 0, `dword_60519C` 0x12345678,
//! `dword_60101C` 2, `dword_621514` 0; `race::set_up_credits_race`,
//! `race::set_up_arcade_cars`, `race::set_start_states`; `dword_71C3E0` 1; "INIT CREDITS -
//! track", the stage, "car", the car kinds logged (level 1, each inside `pusha` /
//! `popa`); `race::load_stage`; `race::roll_credits` with eax 0; `dword_63DB84` 0;
//! `drive::play_race`; `dword_71C3E0` 0; `race::tidy_everything`.
//!
//! Callers: `menu::play_films` (with ebp kept round it), which calls the Rust version directly,
//! and 0x443104. Its calls by place: 0 `set_up_credits_race`, 1 `set_up_arcade_cars`, 2
//! `set_start_states`, 3 `log_write_l1`, 4 `log_int_l1`, 5 `log_int_l1_nl`, 6 `load_stage`, 7
//! `roll_credits`, 8 `play_race`, 9 `tidy_everything`.

use crate::errors::Refused;
use crate::logfile::log_int::log_int;
use crate::race::owner::Race;

const WORDS: [&std::ffi::CStr; 2] = [c"INIT CREDITS - track", c"car"]; // "INIT CREDITS - track", "car"

/// The Rust version: the steps in order. (The original keeps ebp, ecx and edi and leaves
/// esi and edx; no step reads a register of the caller or of an earlier step.)
pub fn credits_race(race: &Race) -> Result<(), Refused> {
    let system = crate::system::owner::system();
    let frame = &race.frame;
    crate::framework::log("race::credits_race: the demonstration starts");
    race.course.numbers.replay_mode.set(0);
    race.shared.replay_file.hold_car_revs.set(0); // g_holdCarRevs
    race.shared.tables.noise_seed.set(0x1234_5678);
    race.physics.split_tables.arcade_kind.set(2);
    race.shared.switches.editor_mode.set(0);
    crate::race::set_up_credits_race::set_up_credits_race(&race.shared)?;
    crate::race::race_cars::set_up_arcade_cars(race)?;
    crate::race::race_shell::set_start_states(race)?;
    race.shared.work.in_credits.set(1);
    let word = |n: usize| &WORDS[n];
    crate::logfile::log_write::log_nul(word(0), false, 1);
    log_int(race.shared.vars.stage_number.get() as i32, false, 1);
    crate::logfile::log_write::log_nul(word(1), false, 1);
    log_int(race.shared.vars.race_car_kinds[0].get() as i32, true, 1);
    crate::race::stage_loading::load_stage(race)?;
    crate::race::frame_text::roll_credits(
        frame,
        0,
        &mut crate::text::draw_text::GameCanvas,
        &mut crate::race::frame_text::credits_tune,
        system,
    )?;
    race.shared.view.after_credits.set(0);
    crate::drive::play_race::play_race(race, crate::scene::owner::scene())?;
    race.shared.work.in_credits.set(0);
    crate::race::race_end::tidy_everything(race)
}
