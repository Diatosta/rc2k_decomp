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

use crate::drive::finish_store::FINISH;
use crate::drive::race_parts::log_line;
use crate::errors::{BadInput, Refused};
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::pointers::GamePtr;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::settings_copy::Place;
use crate::race::settings_copy::var;
use crate::race::state_store::RACE_STATE;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
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
        Var::D60101C => var!(SPLIT_TABLES.arcade_kind),
        Var::D621514 => var!(FRAME_SWITCHES.editor_mode),
        Var::D623854 => var!(RACE_VARS.direction),
        Var::D6235FC => var!(RACE_VARS.car_tyres[0]),
        Var::RaceMode => var!(RACE_VARS.run_kind),
        Var::D6238A4 => var!(RACE_VARS.split),
        Var::RaceCars => var!(RACE_VIEW.players),
        Var::D63DB90 => var!(RACE_VIEW.race_cars),
        Var::D6238A8 => var!(RACE_VARS.settings_words[0]),
        Var::D6238AC => var!(RACE_VARS.settings_words[1]),
        Var::D6238B0 => var!(RACE_VARS.settings_words[2]),
        Var::D6238B4 => var!(RACE_VARS.settings_words[3]),
        Var::D6238B8 => var!(RACE_VARS.settings_words[4]),
        Var::D621890 => var!(FRAME_SWITCHES.view_marks_on),
        Var::D6238CC => var!(RACE_VARS.race_level),
        Var::D6238D0 => var!(RACE_VARS.race_stage),
        Var::D64BB18 => var!(FINISH.no_place),
        Var::D5C2F88 => var!(MARKS_SETTINGS.race_done),
        Var::D606744 => var!(VIEW_VARS.place_last_c),
        Var::D606748 => var!(VIEW_VARS.finish_value),
        Var::D63DB84 => var!(RACE_VIEW.after_credits),
    }
}

/// The settings for the routine (0x4591B0 .. 0x4592FA).
fn set_up() -> Result<(), BadInput> {
    for (at, v) in finish_routine::set_up() {
        match at {
            Set::Var(x) => var(x).set(v),
            // Car block 0 (0x70F3EC) + o.
            Set::Lead(o) => {
                GamePtr::new((crate::race::blocks_store::block_addr(0)).wrapping_add(o))
                    .write(v, "a car's block")?;
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
pub fn finish_race() -> Result<(), Refused> {
    // The stage index is not checked against the seven (a read past them is the original's).
    RACE_VARS.stage_number.set(crate::session::read_on_in(
        &FINISH,
        &FINISH.finish_stages,
        RACE_STATE.results.stage_index.get() as usize,
        "the finish stages",
    )?);
    RACE_VARS.race_car_kinds[0].set(RACE_STATE.results.finish_kind.get());
    CAR_BLOCKS.cars[0]
        .car_kind
        .set_bits(RACE_VARS.race_car_kinds[0].get());
    CAR_BLOCKS.cars[0]
        .start_slot_value
        .set_bits(RACE_STATE.results.finish_player.get());
    log_line(c"----------------------");
    log_line(c"CALLING FINISH ROUTINE");
    log_text(c"[shauntracknum]");
    log_number(RACE_VARS.stage_number.get());
    log_text(c"[shauncarnum1]");
    log_number(RACE_VARS.race_car_kinds[0].get());
    log_text(c"[car1.ctd_carnum]");
    log_number(CAR_BLOCKS.cars[0].start_slot_value.bits());
    set_up()?;
    crate::race::stage_loading::load_stage()?;
    var(finish_routine::BEFORE_RUN).set(0);
    crate::drive::finish_run::finish_run()?;
    // `race::tidy_everything`'s esi "read" is of `finish_run`'s leftover, which has
    // no read point (liveness at 0x459313): a scratch.
    crate::race::race_end::tidy_everything()?;
    for x in finish_routine::AFTER {
        var(x).set(0);
    }
    log_line(c"EXITING FINISH ROUTINE");
    log_line(c"----------------------");
    Ok(())
}
