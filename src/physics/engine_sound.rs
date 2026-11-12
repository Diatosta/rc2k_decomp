//! `physics::engine_sound` (0x43DC72): the work car's sound figures. +0xF08
//! (`engine_speed_scaled`) = speed x `two_and_a_quarter`, and that (x `kilometres_in_mile` with bit 8 of +0x220)
//! `fistp` into +0xAB0; +0x294 = the motion's length (`fsqrt` of z² + x² + y², as the
//! original adds them) x `two_and_a_quarter`; the revs +0x1E4 eased toward the engine speed (at
//! least `idle_engine_speed`, `fistp`, through `scratch`) by 50/256 of the gap (`imul`,
//! `sar`), held to 2000 ..= 12000 (by the sign); then `physics::car_sound_state` and
//! `physics::gear_change_sound`.
//!
//! The x87 run is the arithmetic of the original's instructions over the game's variables
//! (from IDA), in plain Rust over `X` (`math/x87_float.rs`); the instructions are
//! recorded in the tests' `reference_vectors`.
//!
//! Returns: the gap's step, the sound object (`gear_change_sound`)
//!
//! Caller: `physics::car_physics_step` (0x43B905), every physics step. Refuses (before anything)
//! what the two calls refuse.

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::ext_to_int;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::car_sound_state::car_sound_state;
use crate::physics::car_store::Car;
use crate::physics::gear_change_sound::gear_change_sound;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix3::extended;
use rc2k_core::physics::engine::ease_revs;
use rc2k_core::physics::x87::hold_low;

/// 0x43DC72 .. 0x43DCC6: the speed figures (the original's `test` / `jz` is the `if`).
fn run(car: &Car, numbers: &Numbers) {
    run_at(car, numbers);
}

/// The run over the work car's figures:
/// +0x31AEE0 the speed x `two_and_a_quarter`; +0x31AA88 that (x `kilometres_in_mile` with bit 8 of
/// +0x31A1F8) as a whole number; +0x31A26C the length of the motion (+0x31A170 ..) x
/// `two_and_a_quarter`, the squares added as the original adds them (z, then x and y).
fn run_at(car: &Car, numbers: &Numbers) {
    car.engine_speed_scaled
        .st(car.local_velocity[2].ld() * numbers.two_and_a_quarter.ld());
    let speed = car.engine_speed_scaled.ld();
    let flags = car.option_bits.get();
    let scaled = if flags & 0x100 != 0 {
        speed * numbers.kilometres_in_mile.ld()
    } else {
        speed
    };
    car.engine_sound_speed_int.set_bits(to_int(scaled) as u32);
    let (z, x, y) = (
        car.velocity[2].ld(),
        car.velocity[0].ld(),
        car.velocity[1].ld(),
    );
    let sum = z * z + (x * x + y * y);
    car.sound_speed
        .set_bits(st(sum.sqrt() * numbers.two_and_a_quarter.ld()));
}

// Converted: `rc2k_core::physics::engine::ease_revs` and `x87::hold_low` (the speed figures
// and the rounding stay x87).
pub fn engine_sound(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    if shared.view.players.get() == 1 && shared.view_car(0).is_none() {
        return Err(crate::errors::BadInput("a null view block").into());
    }
    run(shared.car(), numbers);
    drawing
        .scene
        .scratch
        .set(x87::loaded(&shared.car().engine_speed).to_bits());
    let floor = shared.physics_tables.idle_engine_speed.bits();
    let held = hold_low(
        f32::from_bits(drawing.scene.scratch.get()),
        f32::from_bits(floor),
    );
    drawing.scene.scratch.set(held.to_bits());
    drawing
        .scene
        .scratch
        .set(ext_to_int(extended(drawing.scene.scratch.get())) as u32);
    let (revs, _) = ease_revs(
        shared.car().engine_sound_revs.get(),
        drawing.scene.scratch.get(),
    );
    shared.car().engine_sound_revs.set(revs);
    { car_sound_state(numbers, matrices, shared) }?;
    gear_change_sound(shared.car())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    #[test]
    fn run_matches_the_assembly() {
        let blocks = vec![Block {
            name: "engine_sound run",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x31A184, In::F),
                    (0x1C5D00, In::F),
                    (0x1C5E08, In::F),
                    (0x31A1F8, In::I),
                    (0x31A178, In::F),
                    (0x31A170, In::F),
                    (0x31A174, In::F),
                ],
                writes: &[(0x31AEE0, Out::F), (0x31AA88, Out::I), (0x31A26C, Out::F)],
            }],
            ulps: 0,
            plain: |_| run_at(race().shared.car(), &race().course.numbers),
        }];
        sparse("physics.engine_sound", &blocks, 4000, 200);
    }
}
