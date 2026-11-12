//! `drive::start_blend` (0x43773D): the checkpoints the work car is past the lead's start
//! (`checkpoint` - `CarsAndReplay.car_blocks.cars[0].checkpoint`, held to 0 .. 100 by the signs), through `scratch` times
//! `start_blend_step` into `start_blend_share`; `start_blend` = `start_share` + that + `pace_push`, held to
//! `zero` .. `one` (0 .. 1). The result is the computer
//! driver's skill share (`driver_start_blend`, read by `drive::read_car_controls` and used by
//! `drive_computer_car` to ease the wanted speed from 30 up to the table's speed:
//! `30 + (wanted - 30) x blend`): the start's grid share (`start_share`, high for the first
//! start) rising with the checkpoints past the lead, plus the pace push (`pace_push`, the arcade
//! opponents' push apart). A player's car under test is given blend 1 (`remote`).
//!
//! Returns: the held count
//!
//! Caller: `drive::read_car_controls` (0x43A486).

use crate::math::x87;
use crate::math::x87::compare_full;
use crate::math::x87_float as bh;
use crate::race::owner::Race;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust core on the variables at `v`: the held count.
pub fn blend_at(race: &Race, v: &[&dyn Cell32]) -> u32 {
    let mut n = get(v, 0).wrapping_sub(get(v, 1));
    if (n as i32) < 0 {
        n = 0;
    }
    if (n.wrapping_sub(100) as i32) >= 0 {
        n = 100;
    }
    put(v, 2, n);
    put(
        v,
        3,
        x87::int_times(n as i32, &race.shared.tables().start_blend_step).to_bits(),
    );
    put(
        v,
        6,
        bh::st(bh::ld(get(v, 4)) + bh::ld(get(v, 3)) + bh::ld(get(v, 5))),
    );
    let (low, high) = (
        race.course.numbers.zero.get(),
        race.course.numbers.one.get(),
    );
    if compare_full(get(v, 6), low) & 0x100 != 0 {
        put(v, 6, x87::loaded(low).to_bits());
    }
    if compare_full(get(v, 6), high) & 0x100 == 0 {
        put(v, 6, x87::loaded(high).to_bits());
    }
    n
}

fn game<'a>(race: &'a Race, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    let shared = &race.shared;
    vec![
        &shared.car().checkpoint,
        &race.car_blocks().cars[0].checkpoint,
        &scene.drawing.scene.scratch,
        &shared.car().start_blend_share,
        &shared.car().start_share,
        &shared.car().pace_push,
        &shared.car().start_blend,
    ]
}

/// (the exact one, for `drive::read_car_controls`).
pub fn start_blend(race: &Race, scene: &crate::scene::owner::Scene) -> u32 {
    blend_at(race, &game(race, scene))
}
