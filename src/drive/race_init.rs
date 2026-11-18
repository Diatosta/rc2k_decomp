//! The race's start after the stage is loaded ("RACEINIT : ..." logged before each part): the
//! best line (`race::store_loaded_best_line`; with fewer than two checkpoints the fatal error
//! 0x80600D1E, unless `editor_mode` is set), the cars' places, objects, game mode, messages, car
//! parts and timers, fifty settling steps (`drive::race_cars_step`, `drive::share_all_cars`,
//! `effects::frame_marks`, `effects::run_frame`), the slots again and the damage messages. Not
//! the race's step loop: a run of fifty steps at the stage's start. Refuses what its parts
//! refuse.

use crate::errors::{FatalError, Refused};
use crate::race::owner::Race;
use rc2k_core::drive::race_steps;

fn indent_by(by: u32) {
    let indent = crate::logfile::write_line::state::INDENT.atomic();
    indent.fetch_add(by, std::sync::atomic::Ordering::Relaxed);
}

fn log(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 1);
}

/// The log indented by 3 round a part.
fn indented(part: &mut dyn FnMut() -> Result<(), Refused>) -> Result<(), Refused> {
    indent_by(race_steps::LOG_INDENT);
    part()?;
    indent_by(race_steps::LOG_INDENT.wrapping_neg());
    Ok(())
}

/// A value handed on to `race::checkpoint_distances` is never read there: that function is
/// reached only with two checkpoints or more.
pub fn race_init(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (sound, scene, effects) = (o.sound, o.scene, o.race.effects.get());
    let race_owner = race;
    crate::race::load_times::mark("setbestline");
    log(c"RACEINIT : setbestline");
    indent_by(race_steps::LOG_INDENT);
    crate::race::best_line::store_loaded_best_line(race, scene)?;
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
    crate::race::load_times::mark("initcarpositions");
    log(c"RACEINIT : initcarpositions");
    indented(&mut || crate::race::car_starts::start_cars(race, scene, &o.net.race_slot))?;
    crate::race::load_times::mark("initobjectsoncourse");
    log(c"RACEINIT : initobjectsoncourse");
    indented(&mut || crate::race::course_data::reset_face_lights(race))?;
    crate::race::load_times::mark("gamemodestuff_init");
    log(c"RACEINIT : gamemodestuff_init");
    indented(&mut || {
        { crate::race::race_cars::set_up_cars(race, scene, &o.net.race_slot) }.map(drop)?;
        // A network race's computer drivers (the cars were all set up as players').
        crate::net::bots::make_drivers(race, o.net);
        Ok(())
    })?;
    crate::race::load_times::mark("messagesystem_init");
    log(c"RACEINIT : messagesystem_init");
    indented(&mut || {
        crate::race::race_init::reset_car_race_state(race)?;
        Ok(())
    })?;
    crate::race::load_times::mark("carparts_init");
    log(c"RACEINIT : carparts_init");
    indented(&mut || {
        crate::race::race_init::clear_race_slots(&race.shared);
        Ok(())
    })?;
    crate::race::load_times::mark("resettimers");
    log(c"RACEINIT : resettimers");
    indented(&mut || {
        {
            crate::race::race_init::reset_car_starts(&race.shared)?;
            crate::race::race_init::seed_car_randoms(&race.shared)?;
        }
        Ok(())
    })?;
    crate::race::load_times::mark("singlecarinit");
    log(c"RACEINIT : singlecarinit");
    indent_by(race_steps::LOG_INDENT);
    for _ in 0..race_steps::SETTLING_STEPS {
        { crate::drive::race_cars_step::race_cars_step(o, race) }?;
        race.effects_settings().other_marks_shown.set(1);
        {
            let view = crate::drive::set_listener::ListenerView::before_any_view(race, scene);
            crate::drive::share_all_cars::share_all_cars(race, sound, scene, view, &o.net.race_slot)
        }?;
        race.effects_settings().other_marks_shown.set(0);
        crate::effects::frame_marks::frame_marks(race, scene, effects)?;
        // No view of the race drawn yet (`EffectsView::before_any_view`).
        let view = crate::effects::run_frame::EffectsView::before_any_view(race, scene);
        crate::effects::run_frame::run_frame(
            &o.scene.scratch.game_scratch().vertices,
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
    crate::race::load_times::mark("resetalldamagemessages");
    log(c"RACEINIT : resetalldamagemessages");
    indented(&mut || {
        crate::race::car_grades::grade_all_cars(&race.shared, scene)?;
        Ok(())
    })?;
    crate::race::load_times::mark("end");
    log(c"RACEINIT : end");
    Ok(())
}
