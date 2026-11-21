//! The championship's finish routine: the stage (`finish_stages[stage_index]` into the stage
//! number), the cars' kind and the finish player logged; one car in mode 3 set up (some thirty
//! settings and block words); the stage loaded (`race::load_stage`); `after_credits` 0;
//! `drive::finish_run`; everything tidied (`race::tidy_everything`); `race_done` and the finish
//! variables 0; logged. Refuses what its calls refuse.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::memory::plain_floats::WordRef;
use crate::memory::session::Cell32;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::drive::finish_routine::{self, Lead, Set, Var};

fn log_text(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, false, 1);
}

fn log_number(v: u32) {
    crate::logfile::log_int::log_int(v as i32, true, 1);
}

/// Writes a finish routine variable: a word of the race's shared state, a drawing store's plain
/// cell (the finish routine's own variables, the view records), or a field of its owner.
fn set_var(race: &Race, v: Var, value: u32) {
    let (vars, view) = (race.shared.vars.get(), race.shared.view.get());
    match v {
        Var::D60101C => race.physics.split_tables_mut().arcade_kind = value,
        Var::D621514 => race.shared.switches.borrow_mut().editor_mode = value,
        Var::D623854 => vars.direction.set(value),
        Var::D6235FC => vars.car_tyres[0].set(value),
        Var::RaceMode => vars.run_kind.set(value),
        Var::D6238A4 => vars.split.set(value),
        Var::RaceCars => view.players.set(value),
        Var::D63DB90 => view.race_cars.set(value),
        Var::D6238A8 => vars.settings_words[0].set(value),
        Var::D6238AC => vars.settings_words[1].set(value),
        Var::D6238B0 => vars.settings_words[2].set(value),
        Var::D6238B4 => vars.settings_words[3].set(value),
        Var::D6238B8 => vars.settings_words[4].set(value),
        Var::D621890 => race.shared.switches.borrow_mut().view_marks_on = value,
        Var::D6238CC => vars.race_level.set(value),
        Var::D6238D0 => vars.race_stage.set(value),
        Var::D64BB18 => race.replay.finish_mut().no_place = value,
        Var::D5C2F88 => race.effects_settings().race_done.set(value),
        // View record 6's first two angles (the finish camera's).
        Var::D606744 => race.frame.view_and_records.get().records[6].angles[0].set(value),
        Var::D606748 => race.frame.view_and_records.get().records[6].angles[1].set(value),
        Var::D63DB84 => view.after_credits.set(value),
    }
}

fn lead_cell(car: &Car, field: Lead) -> WordRef<'_> {
    match field {
        Lead::CameraView => WordRef::Cell(&car.camera_view),
        Lead::GearRatioSetting => WordRef::Cell(&car.gear_ratio_setting),
        Lead::SuspensionStiffness => WordRef::Cell(&car.suspension_stiffness),
        Lead::RideHeight => WordRef::Cell(&car.ride_height),
        Lead::BrakeBalance => WordRef::Cell(&car.brake_balance),
        Lead::SteeringSensitivity => WordRef::Cell(&car.steering_sensitivity),
        Lead::PartsPresent => WordRef::Cell(&car.parts_present),
        Lead::ImpactMark(k) => WordRef::Element(car.impact_octant_marks.cell_at(k as usize)),
    }
}

fn set_up(race: &Race) {
    for (at, v) in finish_routine::set_up() {
        match at {
            Set::Var(x) => set_var(race, x, v),
            Set::Lead(field) => lead_cell(&race.car_blocks().cars[0], field).set_bits(v),
        }
    }
}

pub fn finish_race(race: &'static Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let race_owner = race;
    let replay = &race.replay;
    // The stage index is not checked against the seven (a read past them reads the fields
    // after the table).
    race_owner.shared.vars.get().stage_number.set(
        replay
            .finish()
            .finish_stage(race_owner.shared.state.results.stage_index.get() as usize)?,
    );
    race_owner.shared.vars.get().race_car_kinds[0]
        .set(race_owner.shared.state.results.finish_kind.get());
    race.car_blocks().cars[0]
        .car_kind
        .set(race_owner.shared.vars.get().race_car_kinds[0].get());
    race.car_blocks().cars[0]
        .start_slot_value
        .set(race_owner.shared.state.results.finish_player.get());
    log_line(c"----------------------");
    log_line(c"CALLING FINISH ROUTINE");
    log_text(c"[shauntracknum]");
    log_number(race_owner.shared.vars.get().stage_number.get());
    log_text(c"[shauncarnum1]");
    log_number(race_owner.shared.vars.get().race_car_kinds[0].get());
    log_text(c"[car1.ctd_carnum]");
    log_number(race.car_blocks().cars[0].start_slot_value.get());
    set_up(race);
    crate::race::stage_loading::load_stage(race, o)?;
    set_var(race, finish_routine::BEFORE_RUN, 0);
    crate::drive::finish_run::finish_run(race, o)?;
    // `race::tidy_everything` takes no real input from `finish_run`'s leftover, which has
    // no reader: a scratch.
    crate::race::race_end::tidy_everything(race, o)?;
    for x in finish_routine::AFTER {
        set_var(race, x, 0);
    }
    log_line(c"EXITING FINISH ROUTINE");
    log_line(c"----------------------");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn old_offset(field: Lead) -> usize {
        match field {
            Lead::CameraView => 0x684,
            Lead::GearRatioSetting => 0x28,
            Lead::SuspensionStiffness => 0x34,
            Lead::RideHeight => 0x30,
            Lead::BrakeBalance => 0x20,
            Lead::SteeringSensitivity => 0x3C,
            Lead::PartsPresent => 0x298,
            Lead::ImpactMark(k) => 0x64 + 4 * k as usize,
        }
    }

    #[test]
    fn the_lead_fields_are_the_offsets_they_replace() {
        use crate::physics::car_offsets_1999::{bytes_1999, car_with_1999_bytes, store_1999};
        let pattern = |n: usize| (n * 11 + 1) as u8;
        let named = car_with_1999_bytes(pattern);
        let by_offset = car_with_1999_bytes(pattern);
        let before = bytes_1999(&named);
        for (at, v) in finish_routine::set_up() {
            if let Set::Lead(field) = at {
                lead_cell(&named, field).set_bits(v);
                store_1999(&by_offset, old_offset(field), v);
            }
        }
        assert!(
            bytes_1999(&named) == bytes_1999(&by_offset),
            "the named fields wrote other bytes than the offsets"
        );
        assert_ne!(bytes_1999(&named), before);
    }
}
