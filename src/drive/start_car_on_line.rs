//! The work car put at a start on the line, for the arcade race's opponents (`start_cars_behind`,
//! also at their first start): the start's car number, `driver_kind` 2, the checkpoint and line
//! place, a share lo + ((m - 1) - n) / (m - 1) (hi - lo) (n `start_slot`, m `arcade_cars`, lo / hi
//! `start_share_last` / `start_share_first`) that is the computer driver's pace for that start
//! (see `drive::start_blend`), `race::place_car_on_line`, `race::reset_car_motion`, and the speed
//! `start_speed` turned into the car's speed. Refuses (before anything) a start number above
//! 0x400 (the table `arcade_car_numbers` holds 90), and what the two calls refuse.

use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::drive::start_places;

/// `arcade_car_numbers[start]`: each car's number. The table holds 90; a start past it (the
/// check lets up to 0x400 through; the start is `put_car`'s, dynamic, or `gap_start`'s, held to
/// the count) reads on into the variables after it (kept, the starts not proven).
pub fn car_number(shared: &RaceShared, start: usize) -> Result<u32, BadInput> {
    shared
        .tables()
        .dword_from(crate::race::tables_store::Listed::ArcadeCarNumbers, start)
}

/// `distance` checkpoints on from the lead car.
pub fn start_car_on_line(
    race: &Race,
    distance: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let start = race.shared.tables().start_slot.get();
    start_places::check_start(start)?;
    race.shared
        .car()
        .start_slot_value
        .set(car_number(&race.shared, start as usize)?);
    race.shared
        .car()
        .driver_kind
        .set(start_places::STARTED_MODE);
    let checkpoint =
        start_places::start_checkpoint(race.car_blocks().cars[0].checkpoint.get(), distance);
    for cell in [
        &race.shared.car().checkpoint,
        &race.shared.car().reset_checkpoint,
        &race.shared.car().last_pace_call,
    ] {
        cell.set(checkpoint);
    }
    scene.line_and_shake_mut().line_place = checkpoint;
    scene.line_and_shake_mut().line_fraction = checkpoint;
    let share = start_places::start_share(
        start as i32,
        race.shared.tables().arcade_cars.get() as i32,
        race.course.numbers.one.get(),
        race.shared.tables().start_share_first.get(),
        race.shared.tables().start_share_last.get(),
    );
    race.shared.tables().start_share.set(share);
    race.shared.car().start_share.set(share);
    crate::race::start_place::place_car_on_line(&race.shared, &race.course, scene)?;
    crate::race::car_starts::reset_car_motion(
        &crate::physics::step_inputs::StepInputs::of(race).reset(),
        scene,
    )?;
    race.shared.car().local_velocity.set_at(0, 0.0);
    race.shared.car().local_velocity.set_at(1, 0.0);
    race.shared
        .car()
        .local_velocity
        .set_at(2, x87::loaded(&race.shared.tables().start_speed));
    let m = &race.shared.car().crash_matrix;
    let out = rotate_point(
        m,
        [
            extended(race.shared.car().local_velocity.at(0)),
            extended(race.shared.car().local_velocity.at(1)),
            extended(race.shared.car().local_velocity.at(2)),
        ],
    );
    race.shared.car().velocity.set_at(0, rounded_value(out[0]));
    race.shared.car().velocity.set_at(1, rounded_value(out[1]));
    race.shared.car().velocity.set_at(2, rounded_value(out[2]));
    Ok(())
}
