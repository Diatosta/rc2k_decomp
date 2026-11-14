//! `race::set_up_credits_race`: sets up the credits' race (the demonstration between the films).
//! A stage is drawn from the 36 of `credit_stages` (`math::next_random` x 36, the high dword, held
//! to 0 .. 0x23 by the sign) into `stage_number`, and a car kind from the 23 of `credit_car_kinds`
//! (0 .. 0x16) into `race_car_kinds`; `direction` is 0, `run_kind` 3, and 72 more stores follow
//! (`WRITES`: the first car block's settings, the race's switches).
//!
//! Called by `race::credits_race`. Refuses nothing.

use crate::errors::Refused;
use crate::math::next_random::next_random;
use crate::race::owner::Race;
use crate::race::settings_copy::StepOperand;
use crate::race::settings_copy::{car, car_bits, var};

/// The stores after the two draws, in order (the place, the value).
pub const WRITES: [(StepOperand, u32); 74] = [
    (var!(@race shared.vars.get().direction), 0),
    (var!(@race shared.vars.get().run_kind), 3),
    (car!(0, start_slot_value), 1),
    (car!(0, camera_view), 8),
    (car!(0, gear_ratio_setting), 2),
    (car!(0, suspension_stiffness), 0x80),
    (car!(0, ride_height), 0x80),
    (car!(0, brake_balance), 0x80),
    (car!(0, steering_sensitivity), 0x80),
    (var!(@race shared.vars.get().car_tyres[0]), 3),
    (var!(@race shared.vars.get().car_colours[0]), 1),
    (var!(@race shared.vars.get().mip_setting), 0),
    (var!(@race shared.view.get().race_cars), 1),
    (var!(@race shared.view.get().players), 1),
    (var!(@race shared.vars.get().settings_words[0]), 1),
    (var!(@race shared.vars.get().settings_words[1]), 1),
    (var!(@race shared.vars.get().settings_words[2]), 1),
    (var!(@race shared.vars.get().settings_words[3]), 0),
    (var!(@race shared.vars.get().settings_words[4]), 1),
    (var!(@race shared.switches.get().view_marks_on), 1),
    (var!(@race shared.vars.get().race_level), 0),
    (var!(@race shared.vars.get().race_stage), 0),
    (car_bits!(0, engine_damage), 0),
    (car_bits!(0, exhaust_damage), 0),
    (car_bits!(0, electrics_damage), 0),
    (car_bits!(0, turbo_damage), 0),
    (car_bits!(0, cooling_damage), 0),
    (car_bits!(0, steering_damage), 0),
    (car_bits!(0, clutch_damage), 0),
    (car_bits!(0, differential_damage), 0),
    (car_bits!(0, overall_suspension_damage), 0),
    (car_bits!(0, tyre_wear), 0),
    (car_bits!(0, brake_damage), 0),
    (car_bits!(0, lights_damage), 0),
    (car_bits!(0, gearbox_damage), 0),
    (car_bits!(0, body_damage), 0),
    (car_bits!(0, gear_wear[0]), 0),
    (car_bits!(0, gear_wear[2]), 0),
    (car_bits!(0, gear_wear[3]), 0),
    (car_bits!(0, gear_wear[4]), 0),
    (car_bits!(0, gear_wear[5]), 0),
    (car_bits!(0, gear_wear[6]), 0),
    (car_bits!(0, gear_wear[7]), 0),
    (car_bits!(0, gear_wear[8]), 0),
    (car_bits!(0, wheel_hit_damage[0]), 0),
    (car_bits!(0, wheel_hit_damage[1]), 0),
    (car_bits!(0, wheel_hit_damage[2]), 0),
    (car_bits!(0, wheel_hit_damage[3]), 0),
    (car_bits!(0, body_hit_damage), 0),
    (car_bits!(0, light_damage[0]), 0),
    (car_bits!(0, light_damage[1]), 0),
    (car_bits!(0, light_damage[2]), 0),
    (car_bits!(0, light_damage[3]), 0),
    (car_bits!(0, light_damage[4]), 0),
    (car_bits!(0, light_damage[5]), 0),
    (car_bits!(0, suspension_damage[0]), 0),
    (car_bits!(0, suspension_damage[1]), 0),
    (car_bits!(0, suspension_damage[2]), 0),
    (car_bits!(0, suspension_damage[3]), 0),
    (car_bits!(0, brake_wear_a), 0),
    (car_bits!(0, brake_wear_b), 0),
    (car_bits!(0, tyre_wears[0]), 0),
    (car_bits!(0, tyre_wears[1]), 0),
    (car_bits!(0, tyre_wears[2]), 0),
    (car_bits!(0, tyre_wears[3]), 0),
    (car_bits!(0, impact_octant_marks[0]), 0),
    (car_bits!(0, impact_octant_marks[1]), 0),
    (car_bits!(0, impact_octant_marks[2]), 0),
    (car_bits!(0, impact_octant_marks[3]), 0),
    (car_bits!(0, impact_octant_marks[4]), 0),
    (car_bits!(0, impact_octant_marks[5]), 0),
    (car_bits!(0, impact_octant_marks[6]), 0),
    (car_bits!(0, impact_octant_marks[7]), 0),
    (car!(0, parts_present), 0xFFFF_FFFF),
];

/// A draw (`math::next_random`) as an index among `n`.
fn drawn_index(n: u32) -> u32 {
    rc2k_core::race::credits::drawn_place(next_random(), n)
}

pub fn set_up_credits_race(race: &Race, o: &crate::owners::Owners<'static>) -> Result<(), Refused> {
    let shared = &race.shared;
    use rc2k_core::race::credits::{RACE_KINDS, RACE_STAGES};
    let stage = drawn_index(RACE_STAGES);
    shared
        .vars
        .get()
        .stage_number
        .set(shared.work.get().credit_stages[stage as usize].get());
    let kind = drawn_index(RACE_KINDS);
    shared.vars.get().race_car_kinds[0]
        .set(shared.work.get().credit_car_kinds[kind as usize].get());
    for (at, v) in WRITES {
        at.set_with(o, v);
    }
    Ok(())
}
