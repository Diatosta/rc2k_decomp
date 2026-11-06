//! `drive::start_car_on_line` (0x437613): the work car put at a start on the line:
//! `dword_71AA80` = `dword_605034[dword_6051A0]` (the start's number's entry); the grades'
//! mode `dword_71A234` 2; the checkpoint (`dword_70F484`, the lead car's, + ebx) into
//! `dword_71A070`, `6C`, `74`, `dword_6365E4` and `dword_6365EC`; a share `flt_6051AC` /
//! `flt_71A650` = lo + ((m - 1) - n) / (m - 1) (hi - lo) (n `dword_6051A0`, m
//! `dword_6051B4`, lo / hi `flt_6051A4` / `flt_6051A8`); `race::place_car_on_line`,
//! `race::reset_car_motion`; then the motion: 0, 0 and the speed `flt_6051B0` turned by
//! `g_crashMatrix` (`scene::rotate_point`) into 0x71A170 .. 0x71A178 (`g_carSpeed` the
//! speed). By the look of it a computer car placed at the start of a run (a reading).
//!
//! In:  ebx the checkpoints on from the lead's
//! Out: as `race::reset_car_motion` leaves them; no flags read (`drive::store_car` next)
//!
//! Caller: `drive::start_cars_behind` (0x43703E; four calls). Refuses (before anything) a
//! start number (`dword_6051A0`) above 0x400 (not worked out), and what the two calls
//! refuse.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner_course::Course;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::owner::scene;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::drive::start_places;

/// `dword_605034`[start]: each car's number. The table holds 90; a start past it (the
/// check lets up to 0x400 through) reads on into the variables after it, as the original.
pub fn car_number(start: usize) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        &crate::race::owner::race().shared.tables,
        &crate::race::owner::race().shared.tables.arcade_car_numbers,
        start,
        "the start tables",
    )
}

/// The Rust version: `distance` (ebx) checkpoints on from the lead car. The registers it
/// leaves are read by no caller (`drive::start_cars_behind` sets them again).
// Converted: `rc2k_core::drive::start_places` (the share in plain floats; the motion's
// turn stays x87).
pub fn start_car_on_line(course: &Course, distance: u32) -> Result<(), Refused> {
    let lights = &scene().lights;
    let start = crate::race::owner::race().shared.tables.start_slot.get();
    start_places::check_start(start)?;
    crate::race::owner::race()
        .shared
        .car
        .start_slot_value
        .set(car_number(start as usize)?);
    crate::race::owner::race()
        .shared
        .car
        .driver_kind
        .set(start_places::STARTED_MODE);
    let checkpoint = start_places::start_checkpoint(CAR_BLOCKS.cars[0].checkpoint.get(), distance);
    for cell in [
        &crate::race::owner::race().shared.car.checkpoint,
        &crate::race::owner::race().shared.car.reset_checkpoint,
        &crate::race::owner::race().shared.car.last_pace_call,
    ] {
        cell.set(checkpoint);
    }
    lights.state.line_place.set(checkpoint);
    lights.state.line_fraction.set(checkpoint);
    let share = start_places::start_share(
        start as i32,
        crate::race::owner::race().shared.tables.arcade_cars.get() as i32,
        course.numbers.one.get(),
        crate::race::owner::race().shared.tables.arcade_b.get(),
        crate::race::owner::race().shared.tables.arcade_a.get(),
    )
    .to_bits();
    crate::race::owner::race()
        .shared
        .tables
        .start_share
        .set_bits(share);
    crate::race::owner::race()
        .shared
        .car
        .start_share
        .set_bits(share);
    crate::race::start_place::place_car_on_line(course)?;
    crate::race::car_starts::reset_car_motion()?;
    crate::race::owner::race().shared.car.local_velocity[0].set_bits(0);
    crate::race::owner::race().shared.car.local_velocity[1].set_bits(0);
    crate::race::owner::race().shared.car.local_velocity[2].set_bits(x87::loaded(
        crate::race::owner::race().shared.tables.start_speed.bits(),
    ));
    let m = &crate::race::owner::race().shared.car.crash_matrix;
    let out = rotate_point(
        m,
        [
            extended(crate::race::owner::race().shared.car.local_velocity[0].bits()),
            extended(crate::race::owner::race().shared.car.local_velocity[1].bits()),
            extended(crate::race::owner::race().shared.car.local_velocity[2].bits()),
        ],
    );
    crate::race::owner::race().shared.car.velocity[0].set_bits(rounded(out[0]));
    crate::race::owner::race().shared.car.velocity[1].set_bits(rounded(out[1]));
    crate::race::owner::race().shared.car.velocity[2].set_bits(rounded(out[2]));
    Ok(())
}
