//! `race::credits_race` (0x463666): the credits' race (the demonstration the films play between
//! them): `replay_mode` 0, `g_holdCarRevs` 0, `noise_seed` 0x12345678, `arcade_kind` 2, `editor_mode`
//! 0; `race::set_up_credits_race`, `race::set_up_arcade_cars`, `race::set_start_states`;
//! `in_credits` set; "INIT CREDITS - track", the stage, "car", the car kinds logged (level 1);
//! `race::load_stage`; `race::roll_credits` started; `after_credits` 0; `drive::play_race`;
//! `in_credits` 0; `race::tidy_everything`.
//!
//! Callers: `menu::play_films`, which calls it directly, and the front end's
//! developer entry. Its calls by place: 0 `set_up_credits_race`, 1 `set_up_arcade_cars`, 2
//! `set_start_states`, 3 `log_write_l1`, 4 `log_int_l1`, 5 `log_int_l1_nl`, 6 `load_stage`, 7
//! `roll_credits`, 8 `play_race`, 9 `tidy_everything`.

use crate::errors::Refused;
use crate::logfile::log_int::log_int;
use crate::race::owner::Race;

const WORDS: [&std::ffi::CStr; 2] = [c"INIT CREDITS - track", c"car"]; // "INIT CREDITS - track", "car"

/// The steps in order.
pub fn credits_race(race: &'static Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let frame = &race.frame;
    crate::framework::log("race::credits_race: the demonstration starts");
    race.course.numbers.replay_mode.set(0);
    race.shared.replay_file.hold_car_revs.set(0); // g_holdCarRevs
    race.shared.tables.noise_seed.set(0x1234_5678);
    race.physics.split_tables.arcade_kind.set(2);
    race.shared.switches.editor_mode.set(0);
    crate::race::set_up_credits_race::set_up_credits_race(race, &o)?;
    crate::race::race_cars::set_up_arcade_cars(race)?;
    crate::race::race_shell::set_start_states(race, &o.net.vars)?;
    race.shared.work.in_credits.set(true);
    let word = |n: usize| &WORDS[n];
    crate::logfile::log_write::log_nul(word(0), false, 1);
    log_int(race.shared.vars.stage_number.get() as i32, false, 1);
    crate::logfile::log_write::log_nul(word(1), false, 1);
    log_int(race.shared.vars.race_car_kinds[0].get() as i32, true, 1);
    crate::race::stage_loading::load_stage(race, o)?;
    crate::race::frame_text::roll_credits(
        frame,
        0,
        &mut crate::text::draw_text::GameCanvas,
        &mut || crate::race::frame_text::credits_tune(o),
        o.system,
        o.audio,
    )?;
    race.shared.view.after_credits.set(0);
    crate::drive::play_race::play_race(race, o)?;
    race.shared.work.in_credits.set(false);
    crate::race::race_end::tidy_everything(race, o)
}
