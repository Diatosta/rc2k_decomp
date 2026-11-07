//! `drive::finish_race` (0x459116): the championship's finish routine: the stage
//! (`dword_64BB1C[dword_75B0FC]` into `g_stageNumber`), the cars' kind (`dword_75B104`
//! into `g_raceCarKinds` and the first block's +0x24) and +0xAA8 (`dword_75B100`)
//! logged; one car in mode 3 set up (some thirty settings and block words); the stage
//! loaded (`race::load_stage`); `dword_63DB84` 0; `drive::finish_run`; everything tidied
//! (`race::tidy_everything`); `dword_5C2F88`, `64BB18`, `606744`, `606748` 0; logged.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Callers: `drive::finish_race_0`, `sub_442FEF`. Refuses what its calls refuse.

use crate::drive::race_parts::log_line;
use crate::errors::{BadInput, Refused};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::owner::{RACE, Race, race};
use crate::race::settings_copy::Place;
use crate::race::settings_copy::var;
use crate::race::view_vars_store::ViewVars;
use rc2k_core::drive::finish_routine::{self, Set, Var};

fn log_text(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, false, 1);
}

fn log_number(v: u32) {
    crate::logfile::log_int::log_int(v as i32, true, 1);
}

/// A variable's place.
fn var(v: Var) -> Place {
    match v {
        Var::D60101C => var!(RACE.physics.split_tables.arcade_kind),
        Var::D621514 => var!(crate::race::owner::RACE.shared.switches.editor_mode),
        Var::D623854 => var!(crate::race::owner::RACE.shared.vars.direction),
        Var::D6235FC => var!(crate::race::owner::RACE.shared.vars.car_tyres[0]),
        Var::RaceMode => var!(crate::race::owner::RACE.shared.vars.run_kind),
        Var::D6238A4 => var!(crate::race::owner::RACE.shared.vars.split),
        Var::RaceCars => var!(crate::race::owner::RACE.shared.view.players),
        Var::D63DB90 => var!(crate::race::owner::RACE.shared.view.race_cars),
        Var::D6238A8 => var!(crate::race::owner::RACE.shared.vars.settings_words[0]),
        Var::D6238AC => var!(crate::race::owner::RACE.shared.vars.settings_words[1]),
        Var::D6238B0 => var!(crate::race::owner::RACE.shared.vars.settings_words[2]),
        Var::D6238B4 => var!(crate::race::owner::RACE.shared.vars.settings_words[3]),
        Var::D6238B8 => var!(crate::race::owner::RACE.shared.vars.settings_words[4]),
        Var::D621890 => var!(crate::race::owner::RACE.shared.switches.view_marks_on),
        Var::D6238CC => var!(crate::race::owner::RACE.shared.vars.race_level),
        Var::D6238D0 => var!(crate::race::owner::RACE.shared.vars.race_stage),
        Var::D64BB18 => Place::Var(|| &race().replay.finish.no_place),
        Var::D5C2F88 => var!(MARKS_SETTINGS.race_done),
        // View record 6's first two angles (the finish camera's).
        Var::D606744 => Place::Var(|| &ViewVars::record(6).angles[0]),
        Var::D606748 => Place::Var(|| &ViewVars::record(6).angles[1]),
        Var::D63DB84 => var!(crate::race::owner::RACE.shared.view.after_credits),
    }
}

/// The settings for the routine (0x4591B0 .. 0x4592FA).
fn set_up() -> Result<(), BadInput> {
    for (at, v) in finish_routine::set_up() {
        match at {
            Set::Var(x) => var(x).set(v),
            // Car block 0 (0x70F3EC) + o.
            Set::Lead(o) => {
                crate::pointers::Store::store(&CAR_BLOCKS.cars[0], o as usize, &v.to_le_bytes())?;
            }
        }
    }
    Ok(())
}

/// The Rust version.
// Converted: `rc2k_core::drive::finish_routine`.
///
/// The caller's ecx, which the original hands through `race::load_stage` and
/// `drive::race_init` to `race::checkpoint_distances`, is never read there (that function
/// is reached only with two checkpoints or more; conversion phase 3, 2026-10-24). The
/// registers this leaves are read by no caller (liveness: esi after the menus' calls has
/// no read point).
pub fn finish_race(race: &Race) -> Result<(), Refused> {
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
    CAR_BLOCKS.cars[0]
        .car_kind
        .set_bits(race_owner.shared.vars.race_car_kinds[0].get());
    CAR_BLOCKS.cars[0]
        .start_slot_value
        .set_bits(race_owner.shared.state.results.finish_player.get());
    log_line(c"----------------------");
    log_line(c"CALLING FINISH ROUTINE");
    log_text(c"[shauntracknum]");
    log_number(race_owner.shared.vars.stage_number.get());
    log_text(c"[shauncarnum1]");
    log_number(race_owner.shared.vars.race_car_kinds[0].get());
    log_text(c"[car1.ctd_carnum]");
    log_number(CAR_BLOCKS.cars[0].start_slot_value.bits());
    set_up()?;
    crate::race::stage_loading::load_stage(race)?;
    var(finish_routine::BEFORE_RUN).set(0);
    crate::drive::finish_run::finish_run(race)?;
    // `race::tidy_everything`'s esi "read" is of `finish_run`'s leftover, which has
    // no read point (liveness at 0x459313): a scratch.
    crate::race::race_end::tidy_everything(race)?;
    for x in finish_routine::AFTER {
        var(x).set(0);
    }
    log_line(c"EXITING FINISH ROUTINE");
    log_line(c"----------------------");
    Ok(())
}
