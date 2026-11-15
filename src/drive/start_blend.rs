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
use crate::memory::session::{PlainDword, PlainFloat};
use crate::race::owner::Race;

/// The cells the blend reads and writes.
pub struct BlendCells<'a> {
    /// The work car's checkpoint.
    pub checkpoint: &'a PlainDword,
    /// The lead car's checkpoint.
    pub lead_checkpoint: &'a PlainDword,
    /// The held count.
    pub scratch: &'a crate::memory::session::PlainDword,
    /// `start_blend_share`: the count times `start_blend_step`.
    pub step_share: &'a PlainFloat,
    pub start_share: &'a PlainFloat,
    pub pace_push: &'a PlainFloat,
    /// `start_blend`: the result.
    pub blend: &'a PlainFloat,
}

/// The Rust core on the variables `v`: the held count.
pub fn blend_at(race: &Race, v: &BlendCells<'_>) -> u32 {
    let mut n = v.checkpoint.get().wrapping_sub(v.lead_checkpoint.get());
    if (n as i32) < 0 {
        n = 0;
    }
    if (n.wrapping_sub(100) as i32) >= 0 {
        n = 100;
    }
    v.scratch.set(n);
    v.step_share.set(x87::int_times(
        n as i32,
        &race.shared.tables().start_blend_step,
    ));
    v.blend.set(bh::stf(
        bh::ld(v.start_share.get()) + bh::ld(v.step_share.get()) + bh::ld(v.pace_push.get()),
    ));
    let (low, high) = (
        race.course.numbers.get().zero.get(),
        race.course.numbers.get().one.get(),
    );
    if compare_full(v.blend.get(), low) & 0x100 != 0 {
        v.blend.set(x87::loaded(low));
    }
    if compare_full(v.blend.get(), high) & 0x100 == 0 {
        v.blend.set(x87::loaded(high));
    }
    n
}

fn game<'a>(race: &'a Race, scene: &'a crate::scene::owner::Scene) -> BlendCells<'a> {
    let shared = &race.shared;
    BlendCells {
        checkpoint: &shared.car().checkpoint,
        lead_checkpoint: &race.car_blocks().cars[0].checkpoint,
        scratch: &scene.drawing.scene.get().scratch,
        step_share: &shared.car().start_blend_share,
        start_share: &shared.car().start_share,
        pace_push: &shared.car().pace_push,
        blend: &shared.car().start_blend,
    }
}

/// (the exact one, for `drive::read_car_controls`).
pub fn start_blend(race: &Race, scene: &crate::scene::owner::Scene) -> u32 {
    blend_at(race, &game(race, scene))
}
