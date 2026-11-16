//! The credits' race (the demonstration the films play between them): sets `replay_mode` 0,
//! `hold_car_revs` 0, `noise_seed` 0x12345678, `arcade_kind` 2 and `editor_mode` 0; runs
//! `set_up_credits_race`, `set_up_arcade_cars` and `set_start_states`; sets `in_credits`; logs
//! "INIT CREDITS - track", the stage, "car" and the car kinds (level 1); loads the stage, starts
//! `roll_credits`, sets `after_credits` 0, plays the race, clears `in_credits` and tidies
//! everything.

use crate::errors::Refused;
use crate::logfile::log_int::log_int;
use crate::race::owner::Race;

const WORDS: [&std::ffi::CStr; 2] = [c"INIT CREDITS - track", c"car"];

/// The steps in order.
pub fn credits_race(race: &'static Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let frame = &race.frame;
    crate::logfile::program_log::log("race::credits_race: the demonstration starts");
    race.course.numbers.get().replay_mode.set(0);
    race.shared.replay_file().hold_car_revs.set(0);
    race.shared.tables().noise_seed.set(0x1234_5678);
    race.physics.split_tables().arcade_kind.set(2);
    race.shared.switches.get().editor_mode.set(0);
    crate::race::set_up_credits_race::set_up_credits_race(race, &o)?;
    crate::race::race_cars::set_up_arcade_cars(race)?;
    crate::race::race_shell::set_start_states(race, &o.net.race_slot)?;
    race.shared.work.get().in_credits.set(true);
    let word = |n: usize| &WORDS[n];
    crate::logfile::log_write::log_nul(word(0), false, 1);
    log_int(race.shared.vars.get().stage_number.get() as i32, false, 1);
    crate::logfile::log_write::log_nul(word(1), false, 1);
    log_int(
        race.shared.vars.get().race_car_kinds[0].get() as i32,
        true,
        1,
    );
    crate::race::stage_loading::load_stage(race, o)?;
    crate::race::frame_text::roll_credits(
        crate::strings::append_string_by_id::StringOwners::of(o),
        frame,
        0,
        &mut crate::text::draw_text::GameCanvas::new(o.gfx),
        &mut || crate::race::frame_text::credits_tune(o),
        o.system,
        o.audio,
    )?;
    race.shared.view.get().after_credits.set(0);
    crate::drive::play_race::play_race(race, o)?;
    race.shared.work.get().in_credits.set(false);
    crate::race::race_end::tidy_everything(race, o)
}
