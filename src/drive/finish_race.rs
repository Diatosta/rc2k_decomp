//! The championship's finish routine: the stage (`finish_stages[stage_index]` into the stage
//! number), the cars' kind and the finish player logged; one car in mode 3 set up (some thirty
//! settings and block words); the stage loaded (`race::load_stage`); `after_credits` 0;
//! `drive::finish_run`; everything tidied (`race::tidy_everything`); `race_done` and the finish
//! variables 0; logged. Refuses what its calls refuse.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::session::Cell32;
use rc2k_core::drive::finish_routine::{self, Lead, Set, Var};

fn log_text(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, false, 1);
}

fn log_number(v: u32) {
    crate::logfile::log_int::log_int(v as i32, true, 1);
}

/// A variable's place: a cell of the race's shared state, or a drawing store's plain cell (the
/// finish routine's own variables, the view records).
enum Place<'a> {
    Shared(&'a dyn crate::session::DwordLike),
    Plain(&'a crate::session::PlainDword),
}

impl Place<'_> {
    fn set(&self, value: u32) {
        match self {
            Place::Shared(cell) => cell.set(value),
            Place::Plain(cell) => cell.set(value),
        }
    }
}

fn var(race: &Race, v: Var) -> Place<'_> {
    Place::Shared(match v {
        Var::D60101C => &race.physics.split_tables().arcade_kind,
        Var::D621514 => return Place::Plain(&race.shared.switches.get().editor_mode),
        Var::D623854 => &race.shared.vars.get().direction,
        Var::D6235FC => &race.shared.vars.get().car_tyres[0],
        Var::RaceMode => &race.shared.vars.get().run_kind,
        Var::D6238A4 => &race.shared.vars.get().split,
        Var::RaceCars => &race.shared.view.get().players,
        Var::D63DB90 => &race.shared.view.get().race_cars,
        Var::D6238A8 => &race.shared.vars.get().settings_words[0],
        Var::D6238AC => &race.shared.vars.get().settings_words[1],
        Var::D6238B0 => &race.shared.vars.get().settings_words[2],
        Var::D6238B4 => &race.shared.vars.get().settings_words[3],
        Var::D6238B8 => &race.shared.vars.get().settings_words[4],
        Var::D621890 => return Place::Plain(&race.shared.switches.get().view_marks_on),
        Var::D6238CC => &race.shared.vars.get().race_level,
        Var::D6238D0 => &race.shared.vars.get().race_stage,
        Var::D64BB18 => return Place::Plain(&race.replay.finish.get().no_place),
        Var::D5C2F88 => &race.effects_settings().race_done,
        // View record 6's first two angles (the finish camera's).
        Var::D606744 => {
            return Place::Plain(&race.frame.view_and_records.get().records[6].angles[0]);
        }
        Var::D606748 => {
            return Place::Plain(&race.frame.view_and_records.get().records[6].angles[1]);
        }
        Var::D63DB84 => &race.shared.view.get().after_credits,
    })
}

fn lead_cell(car: &Car, field: Lead) -> &dyn Cell32 {
    match field {
        Lead::CameraView => &car.camera_view,
        Lead::GearRatioSetting => &car.gear_ratio_setting,
        Lead::SuspensionStiffness => &car.suspension_stiffness,
        Lead::RideHeight => &car.ride_height,
        Lead::BrakeBalance => &car.brake_balance,
        Lead::SteeringSensitivity => &car.steering_sensitivity,
        Lead::PartsPresent => &car.parts_present,
        Lead::ImpactMark(k) => &car.impact_octant_marks[k as usize],
    }
}

fn set_up(race: &Race) {
    for (at, v) in finish_routine::set_up() {
        match at {
            Set::Var(x) => var(race, x).set(v),
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
            .finish
            .get()
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
