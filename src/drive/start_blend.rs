//! The checkpoints the work car is past the lead's start
//! (`checkpoint` - `CarBlocks.cars[0].checkpoint`, held to 0 .. 100 by the signs), through `scratch` times
//! `start_blend_step` into `start_blend_share`; `start_blend` = `start_share` + that + `pace_push`, held to
//! `zero` .. `one` (0 .. 1). The result is the computer
//! driver's skill share (`driver_start_blend`, read by `drive::read_car_controls` and used by
//! `drive_computer_car` to ease the wanted speed from 30 up to the table's speed:
//! `30 + (wanted - 30) x blend`): the start's grid share (`start_share`, high for the first
//! start) rising with the checkpoints past the lead, plus the pace push (`pace_push`, the arcade
//! opponents' push apart). A player's car under test is given blend 1 (`remote`).
//!
//! Gives the held count.

use crate::math::x87;
use crate::math::x87::compare_full;
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
use crate::race::owner::Race;

/// The Rust core on the work car `car` and the lead car's checkpoint: the held count, which is
/// also left in `scratch`.
fn blend_at(race: &Race, car: &mut Car, lead_checkpoint: u32, scratch: &mut u32) -> u32 {
    let mut n = car.checkpoint.wrapping_sub(lead_checkpoint);
    if (n as i32) < 0 {
        n = 0;
    }
    if (n.wrapping_sub(100) as i32) >= 0 {
        n = 100;
    }
    *scratch = n;
    car.start_blend_share = x87::int_times(n as i32, &race.shared.tables().start_blend_step);
    car.start_blend = bh::stf(
        bh::ld(car.start_share.to_bits())
            + bh::ld(car.start_blend_share.to_bits())
            + bh::ld(car.pace_push.to_bits()),
    );
    let (low, high) = (
        race.course.numbers.zero.get(),
        race.course.numbers.one.get(),
    );
    if compare_full(car.start_blend, low) & 0x100 != 0 {
        car.start_blend = x87::loaded(low);
    }
    if compare_full(car.start_blend, high) & 0x100 == 0 {
        car.start_blend = x87::loaded(high);
    }
    n
}

/// (the exact one, for `drive::read_car_controls`).
pub fn start_blend(race: &Race, scene: &crate::scene::owner::Scene) -> u32 {
    let mut scratch = scene.draw_work().scratch;
    let lead_checkpoint = race.car_blocks().cars[0].checkpoint;
    let n = blend_at(
        race,
        &mut race.shared.car_mut(),
        lead_checkpoint,
        &mut scratch,
    );
    scene.draw_work_mut().scratch = scratch;
    n
}
