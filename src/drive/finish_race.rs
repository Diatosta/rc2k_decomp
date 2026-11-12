//! `drive::finish_race` (0x459116): the championship's finish routine: the stage
//! (`finish_stages`[RaceState.results.stage_index]` into `g_stageNumber`), the cars' kind (`RaceState.results.finish_kind` into
//! `g_raceCarKinds` and the first block's +0x24) and +0xAA8 (`RaceState.results.finish_player`) logged; one car in
//! mode 3 set up (some thirty settings and block words); the stage loaded (`race::load_stage`);
//! `after_credits` 0; `drive::finish_run`; everything tidied (`race::tidy_everything`);
//! `race_done`, `64BB18`, `606744`, `606748` 0; logged.
//!
//! Callers: `drive::finish_race_0`, 0x442FEF. Refuses what its calls refuse.

use crate::drive::race_parts::log_line;
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::drive::finish_routine::{self, Set, Var};

fn log_text(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, false, 1);
}

fn log_number(v: u32) {
    crate::logfile::log_int::log_int(v as i32, true, 1);
}

/// A variable's place.
fn var(race: &Race, v: Var) -> &Dword {
    match v {
        Var::D60101C => &race.physics.split_tables.arcade_kind,
        Var::D621514 => &race.shared.switches.editor_mode,
        Var::D623854 => &race.shared.vars.direction,
        Var::D6235FC => &race.shared.vars.car_tyres[0],
        Var::RaceMode => &race.shared.vars.run_kind,
        Var::D6238A4 => &race.shared.vars.split,
        Var::RaceCars => &race.shared.view.players,
        Var::D63DB90 => &race.shared.view.race_cars,
        Var::D6238A8 => &race.shared.vars.settings_words[0],
        Var::D6238AC => &race.shared.vars.settings_words[1],
        Var::D6238B0 => &race.shared.vars.settings_words[2],
        Var::D6238B4 => &race.shared.vars.settings_words[3],
        Var::D6238B8 => &race.shared.vars.settings_words[4],
        Var::D621890 => &race.shared.switches.view_marks_on,
        Var::D6238CC => &race.shared.vars.race_level,
        Var::D6238D0 => &race.shared.vars.race_stage,
        Var::D64BB18 => &race.replay.finish.no_place,
        Var::D5C2F88 => &race.effects_settings().race_done,
        // View record 6's first two angles (the finish camera's).
        Var::D606744 => &race.frame.view_and_records.records[6].angles[0],
        Var::D606748 => &race.frame.view_and_records.records[6].angles[1],
        Var::D63DB84 => &race.shared.view.after_credits,
    }
}

/// The settings for the routine (0x4591B0 .. 0x4592FA).
fn set_up(race: &Race) -> Result<(), BadInput> {
    for (at, v) in finish_routine::set_up() {
        match at {
            Set::Var(x) => var(race, x).set(v),
            // Car block 0 (0x70F3EC) + o.
            Set::Lead(o) => {
                crate::cells::Store::store(
                    &race.car_blocks().cars[0],
                    o as usize,
                    &v.to_le_bytes(),
                )?;
            }
        }
    }
    Ok(())
}

// Converted: `rc2k_core::drive::finish_routine`.
pub fn finish_race(race: &'static Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let race_owner = race;
    let replay = &race.replay;
    // The stage index is not checked against the seven (a read past them is the original's).
    race_owner
        .shared
        .vars
        .stage_number
        .set(crate::session::read_on_in(
            &replay.finish,
            &replay.finish.finish_stages,
            race_owner.shared.state.results.stage_index.get() as usize,
            "the finish stages",
        )?);
    race_owner.shared.vars.race_car_kinds[0].set(race_owner.shared.state.results.finish_kind.get());
    race.car_blocks().cars[0]
        .car_kind
        .set(race_owner.shared.vars.race_car_kinds[0].get());
    race.car_blocks().cars[0]
        .start_slot_value
        .set(race_owner.shared.state.results.finish_player.get());
    log_line(c"----------------------");
    log_line(c"CALLING FINISH ROUTINE");
    log_text(c"[shauntracknum]");
    log_number(race_owner.shared.vars.stage_number.get());
    log_text(c"[shauncarnum1]");
    log_number(race_owner.shared.vars.race_car_kinds[0].get());
    log_text(c"[car1.ctd_carnum]");
    log_number(race.car_blocks().cars[0].start_slot_value.get());
    set_up(race)?;
    crate::race::stage_loading::load_stage(race, o)?;
    var(race, finish_routine::BEFORE_RUN).set(0);
    crate::drive::finish_run::finish_run(race, o)?;
    // `race::tidy_everything` takes no real input from `finish_run`'s leftover, which has
    // no reader: a scratch.
    crate::race::race_end::tidy_everything(race, o)?;
    for x in finish_routine::AFTER {
        var(race, x).set(0);
    }
    log_line(c"EXITING FINISH ROUTINE");
    log_line(c"----------------------");
    Ok(())
}
