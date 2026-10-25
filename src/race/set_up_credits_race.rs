//! `race::set_up_credits_race` (0x463711): the credits' race (the demonstration between
//! the films) set up: a stage drawn from the 36 of `dword_71C3E4` (`math::next_random`
//! x 36, the high dword, held to 0 .. 0x23 by the sign) into `g_stageNumber`, a car kind
//! from the 23 of `dword_71C474` (x 23, 0 .. 0x16) into `g_raceCarKinds`;
//! `dword_623854` 0, `dword_62389C` 3, and 72 more stores (`WRITES`: the first car
//! block's settings, the race's switches, -1 into +0x298).
//!
//! In:  nothing
//! Out: eax 3, ebx 0x17, edx the car kind's index; the rest kept; no flags read (a call
//!      follows)
//!
//! Caller: `race::credits_race`, which calls the Rust version directly. Refuses nothing.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::math::next_random::next_random;
use crate::race::settings_copy::Place::{self, Addr};
use crate::race::settings_copy::{car, car_bits, var};
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;

const STAGE: usize = 0x623850; // g_stageNumber
const CAR_KINDS: usize = 0x623858; // g_raceCarKinds

/// The stores after the two draws, in order (the place, the value).
pub const WRITES: [(Place, u32); 74] = [
    (var!(RACE_VARS.direction), 0),
    (var!(RACE_VARS.run_kind), 3),
    (car!(0, start_slot_value), 1),
    (car!(0, camera_view), 8),
    (car!(0, gear_ratio_setting), 2),
    (car!(0, suspension_stiffness), 0x80),
    (car!(0, ride_height), 0x80),
    (car!(0, brake_balance), 0x80),
    (car!(0, steering_sensitivity), 0x80),
    (var!(RACE_VARS.car_tyres[0]), 3),
    (var!(RACE_VARS.car_colours[0]), 1),
    (var!(RACE_VARS.mip_setting), 0),
    (Addr(0x63DB90), 1),
    (Addr(0x63DB94), 1),
    (var!(RACE_VARS.settings_words[0]), 1),
    (var!(RACE_VARS.settings_words[1]), 1),
    (var!(RACE_VARS.settings_words[2]), 1),
    (var!(RACE_VARS.settings_words[3]), 0),
    (var!(RACE_VARS.settings_words[4]), 1),
    (Addr(0x621890), 1),
    (var!(RACE_VARS.race_level), 0),
    (var!(RACE_VARS.race_stage), 0),
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
    (car_bits!(0, part_hit_damage[0]), 0),
    (car_bits!(0, part_hit_damage[1]), 0),
    (car_bits!(0, part_hit_damage[2]), 0),
    (car_bits!(0, part_hit_damage[3]), 0),
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

/// A draw (`math::next_random`) as a place among n.
fn drawn_index(n: u32) -> u32 {
    rc2k_core::race::credits::drawn_place(unsafe { next_random() }, n)
}

/// The Rust version. (The original leaves eax 3, ebx 0x17 and edx the car kind drawn;
/// nothing reads them.)
pub fn set_up_credits_race() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::credits::drawn_place`.
    use rc2k_core::race::credits::{RACE_KINDS, RACE_STAGES};
    unsafe {
        let stage = drawn_index(RACE_STAGES);
        game_write(rva(STAGE), RACE_WORK.credit_stages[stage as usize].get());
        let kind = drawn_index(RACE_KINDS);
        game_write(
            rva(CAR_KINDS),
            RACE_WORK.credit_car_kinds[kind as usize].get(),
        );
        for (at, v) in WRITES {
            at.set(v);
        }
        Ok(())
    }
}
