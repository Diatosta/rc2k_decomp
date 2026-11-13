//! `drive::race_init` (0x4462B9, three chunks): the race's start after the stage is
//! loaded ("RACEINIT : ..." logged before each part, the log indented by 3 round it):
//! the best line (`race::build_best_line`); with fewer than two checkpoints
//! (`g_checkpointCount`, by the sign of the `cmp`) the fatal error 0x80600D1E "Race
//! initialisation failed - no best line" (its third chunk, 0x4D917B), unless
//! `editor_mode` is set (then it returns); the cars' places (`race::start_cars`), the
//! objects (`race::reset_face_lights`), the game mode (`race::set_up_cars`), the messages
//! (`race::reset_car_race_state`), the car parts (`race::clear_race_slots`), the timers
//! (`race::reset_car_starts`, `race::seed_car_randoms`); fifty settling steps (in `pusha`
//! / `popa`: `drive::race_cars_step`; `drive::share_all_cars` with `other_marks_shown` 1;
//! `effects::frame_marks`, `effects::run_frame`); the slots again; the damage messages
//! (`race::grade_all_cars`); "RACEINIT : end". Its second chunk (0x40763C ..) is the
//! fatal-error reporter's, shared, not entered from here.
//!
//! Not the race's step loop (that runs elsewhere, a step per frame): a run of fifty steps at
//! the stage's start.
//!
//! Caller: `race::load_stage` (0x44412D). Refuses what its parts refuse; raises 0x80600D1E
//! as the original.

use crate::errors::{FatalError, Refused};
use crate::race::owner::Race;
use rc2k_core::drive::race_steps;

/// `g_logIndent` changed by `by` (wrapping, as the original's `add`).
fn indent_by(by: u32) {
    let indent = crate::logfile::write_line::state::INDENT.atomic();
    indent.fetch_add(by, std::sync::atomic::Ordering::Relaxed);
}

/// A "RACEINIT : ..." line (level 1, a line break).
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

/// The registers the original leaves are read by no caller. A value the
/// caller hands on to `race::checkpoint_distances` is never read there: that function
/// is reached only with two checkpoints or more.
// Converted: `rc2k_core::drive::race_steps::{no_best_line, NO_BEST_LINE, SETTLING_STEPS,
// LOG_INDENT}`.
pub fn race_init(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (sound, scene, effects, net) = (o.sound, o.scene, o.effects, o.net);
    let race_owner = race;
    log(c"RACEINIT : setbestline");
    indent_by(race_steps::LOG_INDENT);
    crate::race::best_line::build_best_line(race, scene)?;
    if race_steps::no_best_line(race_owner.shared.vars.get().checkpoint_count.get()) {
        if race_owner.shared.switches.get().editor_mode.get() != 0 {
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
    indented(&mut || crate::race::car_starts::start_cars(race, scene, net))?;
    log(c"RACEINIT : initobjectsoncourse");
    indented(&mut || crate::race::course_data::reset_face_lights(race))?;
    log(c"RACEINIT : gamemodestuff_init");
    indented(&mut || { crate::race::race_cars::set_up_cars(race, scene, net) }.map(drop))?;
    log(c"RACEINIT : messagesystem_init");
    indented(&mut || {
        crate::race::race_init::reset_car_race_state(race)?;
        Ok(())
    })?;
    log(c"RACEINIT : carparts_init");
    indented(&mut || {
        crate::race::race_init::clear_race_slots(&race.shared);
        Ok(())
    })?;
    log(c"RACEINIT : resettimers");
    indented(&mut || {
        {
            crate::race::race_init::reset_car_starts(&race.shared)?;
            crate::race::race_init::seed_car_randoms(&race.shared)?;
        }
        Ok(())
    })?;
    log(c"RACEINIT : singlecarinit");
    indent_by(race_steps::LOG_INDENT);
    for _ in 0..race_steps::SETTLING_STEPS {
        { crate::drive::race_cars_step::race_cars_step(o, race) }?;
        race.effects_settings().other_marks_shown.set(1);
        {
            let view = crate::drive::set_listener::ListenerView::before_any_view(race);
            crate::drive::share_all_cars::share_all_cars(race, sound, scene, net, view)
        }?;
        race.effects_settings().other_marks_shown.set(0);
        crate::effects::frame_marks::frame_marks(race, scene, effects)?;
        // No view of the race drawn yet (`EffectsView::before_any_view`).
        let view = crate::effects::run_frame::EffectsView::before_any_view(race);
        crate::effects::run_frame::run_frame(
            &crate::workers::game_scratch().vertices,
            race,
            scene,
            effects,
            o.system,
            o.gfx,
            sound,
            view,
        )?;
    }
    crate::race::race_init::clear_race_slots(&race.shared);
    indent_by(race_steps::LOG_INDENT.wrapping_neg());
    log(c"RACEINIT : resetalldamagemessages");
    indented(&mut || {
        crate::race::car_grades::grade_all_cars(&race.shared, scene)?;
        Ok(())
    })?;
    log(c"RACEINIT : end");
    Ok(())
}
