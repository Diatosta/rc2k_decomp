//! `drive::start_car_on_line` (0x437613): the work car put at a start on the line: `dword_71AA80` =
//! `dword_605034[dword_6051A0]` (the start's number's entry); the grades' mode `dword_71A234` 2;
//! the checkpoint into `dword_71A070`, `6C`, `74`, `dword_6365E4` and `dword_6365EC`; a share
//! `flt_6051AC` / `flt_71A650` = lo + ((m - 1) - n) / (m - 1) (hi - lo) (n `dword_6051A0`, m
//! `dword_6051B4`, lo / hi `flt_6051A4` / `flt_6051A8`); `race::place_car_on_line`,
//! `race::reset_car_motion`; then the motion: 0, 0 and the speed `flt_6051B0` turned by
//! `g_crashMatrix` (`scene::rotate_point`) into 0x71A170 .. 0x71A178 (`g_carSpeed` the speed). The
//! share is the computer driver's pace for that start (best start = `start_share_first`, last =
//! `start_share_last`; see `drive::start_blend`). Used by `start_cars_behind` for the arcade race's
//! opponents (also at their first start).
//!
//! Takes: the checkpoints on from the lead's
//!
//! Caller: `drive::start_cars_behind` (0x43703E; four calls). Refuses (before anything) a
//! start number (`dword_6051A0`) above 0x400 (the table `dword_605034` holds 90; the check is the original's), and what the two calls
//! refuse.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use rc2k_core::drive::start_places;

/// `dword_605034`[start]: each car's number. The table holds 90; a start past it (the
/// check lets up to 0x400 through) reads on into the variables after it, as the original.
pub fn car_number(shared: &RaceShared, start: usize) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        &shared.tables,
        &shared.tables.arcade_car_numbers,
        start,
        "the start tables",
    )
}

/// `distance` checkpoints on from the lead car.
// Converted: `rc2k_core::drive::start_places` (the share in plain floats; the motion's
// turn stays x87).
pub fn start_car_on_line(
    race: &Race,
    distance: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let lights = &scene.lights;
    let start = race.shared.tables.start_slot.get();
    start_places::check_start(start)?;
    race.shared
        .car
        .start_slot_value
        .set(car_number(&race.shared, start as usize)?);
    race.shared.car.driver_kind.set(start_places::STARTED_MODE);
    let checkpoint = start_places::start_checkpoint(CAR_BLOCKS.cars[0].checkpoint.get(), distance);
    for cell in [
        &race.shared.car.checkpoint,
        &race.shared.car.reset_checkpoint,
        &race.shared.car.last_pace_call,
    ] {
        cell.set(checkpoint);
    }
    lights.state.line_place.set(checkpoint);
    lights.state.line_fraction.set(checkpoint);
    let share = start_places::start_share(
        start as i32,
        race.shared.tables.arcade_cars.get() as i32,
        race.course.numbers.one.get(),
        race.shared.tables.start_share_first.get(),
        race.shared.tables.start_share_last.get(),
    )
    .to_bits();
    race.shared.tables.start_share.set_bits(share);
    race.shared.car.start_share.set_bits(share);
    crate::race::start_place::place_car_on_line(race, &race.course, scene)?;
    crate::race::car_starts::reset_car_motion(race)?;
    race.shared.car.local_velocity[0].set_bits(0);
    race.shared.car.local_velocity[1].set_bits(0);
    race.shared.car.local_velocity[2].set_bits(x87::loaded(race.shared.tables.start_speed.bits()));
    let m = &race.shared.car.crash_matrix;
    let out = rotate_point(
        m,
        [
            extended(race.shared.car.local_velocity[0].bits()),
            extended(race.shared.car.local_velocity[1].bits()),
            extended(race.shared.car.local_velocity[2].bits()),
        ],
    );
    race.shared.car.velocity[0].set_bits(rounded(out[0]));
    race.shared.car.velocity[1].set_bits(rounded(out[1]));
    race.shared.car.velocity[2].set_bits(rounded(out[2]));
    Ok(())
}
