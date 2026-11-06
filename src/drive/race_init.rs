//! `drive::race_init` (0x4462B9, three chunks): the race's start after the stage is
//! loaded ("RACEINIT : ..." logged before each part, the log indented by 3 round it):
//! the best line (`race::build_best_line`); with fewer than two checkpoints
//! (`g_checkpointCount`, by the sign of the `cmp`) the fatal error 0x80600D1E "Race
//! initialisation failed - no best line" (its third chunk, 0x4D917B), unless
//! `dword_621514` is set (then it returns); the cars' places (`race::start_cars`), the
//! objects (`race::reset_face_lights`), the game mode (`race::set_up_cars`), the messages
//! (`race::reset_car_race_state`), the car parts (`race::clear_race_slots`), the timers
//! (`race::reset_car_starts`, `race::seed_car_randoms`); fifty settling steps (in `pusha`
//! / `popa`: `drive::race_cars_step`; `drive::share_all_cars` with `dword_5C2F94` 1;
//! `effects::frame_marks`, `effects::run_frame`); the slots again; the damage messages
//! (`race::grade_all_cars`); "RACEINIT : end". Its second chunk (0x40763C ..) is the
//! fatal-error reporter's, shared, not entered from here.
//!
//! Not the race's step loop (that runs elsewhere, a step per frame): a run of fifty steps at
//! the stage's start.
//!
//! In:  nothing
//! Out: the registers as the parts leave them (ecx 0x32 after the steps); no flags read
//!
//! Caller: `race::load_stage` (0x44412D). Refuses what its parts refuse; raises 0x80600D1E
//! as the original.

use crate::errors::{FatalError, Refused};
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::drive::race_steps;

/// `g_logIndent` changed by `by` (wrapping, as the original's `add`).
fn indent_by(by: u32) {
    let indent = crate::logfile::write_line::state::INDENT.atomic();
    indent.fetch_add(by, std::sync::atomic::Ordering::Relaxed);
}

/// A "RACEINIT : ..." line (level 1, a line break; in `pusha` / `popa`).
fn log(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 1);
}

/// The log indented round a part (`pushf`; +- 3; `popf`).
fn indented(part: &mut dyn FnMut() -> Result<(), Refused>) -> Result<(), Refused> {
    indent_by(race_steps::LOG_INDENT);
    part()?;
    indent_by(race_steps::LOG_INDENT.wrapping_neg());
    Ok(())
}

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx and edx
/// after `race::load_stage`'s call have no read point). The caller's ecx, which the
/// original hands on to `race::checkpoint_distances`, is never read there: that function
/// is reached only with two checkpoints or more (conversion phase 3, 2026-10-24).
// Converted: `rc2k_core::drive::race_steps::{no_best_line, NO_BEST_LINE, SETTLING_STEPS,
// LOG_INDENT}`.
pub fn race_init() -> Result<(), Refused> {
    log(c"RACEINIT : setbestline");
    indent_by(race_steps::LOG_INDENT);
    crate::race::best_line::build_best_line()?;
    if race_steps::no_best_line(RACE_VARS.checkpoint_count.get()) {
        if FRAME_SWITCHES.editor_mode.get() != 0 {
            return Ok(());
        }
        return Err(FatalError {
            code: race_steps::NO_BEST_LINE,
            message: "Race initialisation failed - no best line",
        }
        .into());
    }
    indent_by(race_steps::LOG_INDENT.wrapping_neg());
    log(c"RACEINIT : initcarpositions");
    indented(&mut || crate::race::car_starts::start_cars())?;
    log(c"RACEINIT : initobjectsoncourse");
    indented(&mut || crate::race::course_data::reset_face_lights())?;
    log(c"RACEINIT : gamemodestuff_init");
    indented(&mut || { crate::race::race_cars::set_up_cars() }.map(drop))?;
    log(c"RACEINIT : messagesystem_init");
    indented(&mut || {
        crate::race::race_init::reset_car_race_state()?;
        Ok(())
    })?;
    log(c"RACEINIT : carparts_init");
    indented(&mut || {
        crate::race::race_init::clear_race_slots();
        Ok(())
    })?;
    log(c"RACEINIT : resettimers");
    indented(&mut || {
        {
            crate::race::race_init::reset_car_starts()?;
            crate::race::race_init::seed_car_randoms()?;
        }
        Ok(())
    })?;
    log(c"RACEINIT : singlecarinit");
    indent_by(race_steps::LOG_INDENT);
    for _ in 0..race_steps::SETTLING_STEPS {
        { crate::drive::race_cars_step::race_cars_step() }?;
        MARKS_SETTINGS.other_marks_shown.set(1);
        { crate::drive::share_all_cars::share_all_cars() }?;
        MARKS_SETTINGS.other_marks_shown.set(0);
        crate::effects::frame_marks::frame_marks(crate::effects::owner::effects())?;
        crate::effects::run_frame::run_frame()?;
    }
    crate::race::race_init::clear_race_slots();
    indent_by(race_steps::LOG_INDENT.wrapping_neg());
    log(c"RACEINIT : resetalldamagemessages");
    indented(&mut || {
        crate::race::car_grades::grade_all_cars()?;
        Ok(())
    })?;
    log(c"RACEINIT : end");
    Ok(())
}
