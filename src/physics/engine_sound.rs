//! The work car's sound figures. `engine_speed_scaled` = speed x `two_and_a_quarter`, and that
//! (times `kilometres_in_mile` with bit 8 of the car's flags) rounded into its integer figure;
//! the motion's length (the square root of z² + x² + y², added in that order) times
//! `two_and_a_quarter`; the revs eased toward the engine speed (at least `idle_engine_speed`,
//! rounded, through `scratch`) by 50/256 of the gap and held to 2000 ..= 12000; then
//! `physics::car_sound_state` and `physics::gear_change_sound`.
//!
//! The x87 run is plain Rust over `X` (`math/x87_float.rs`).
//!
//! Returns the gap's step, the sound object (`gear_change_sound`).
//!
//! Refuses (before anything) what the two calls refuse.

use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::extended;
use crate::math::x87;
use crate::math::x87::ext_to_int;
use crate::math::x87_float::stf;
use crate::math::x87_float::to_int;
use crate::physics::car_sound_state::car_sound_state;
use crate::physics::car_store::Car;
use crate::physics::gear_change_sound::gear_change_sound;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::engine::ease_revs;
use rc2k_core::physics::x87::hold_low;

/// The speed figures.
fn run(car: &Car, numbers: &Numbers) {
    run_at(car, numbers);
}

/// The run over the work car's figures:
/// `engine_speed_scaled` = the speed x `two_and_a_quarter`; its whole-number figure = that (x
/// `kilometres_in_mile` with bit 8 of the car's flags); the length of the motion x
/// `two_and_a_quarter`, the squares added in the order z, then x and y.
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
    car.engine_sound_speed_int.set(to_int(scaled) as u32);
    let (z, x, y) = (
        car.velocity[2].ld(),
        car.velocity[0].ld(),
        car.velocity[1].ld(),
    );
    let sum = z * z + (x * x + y * y);
    car.sound_speed
        .set(stf(sum.sqrt() * numbers.two_and_a_quarter.ld()));
}

// Core: `rc2k_core::physics::engine::ease_revs` and `x87::hold_low` (the speed figures
// and the rounding stay x87).
pub(super) fn engine_sound(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    shared: &RaceShared,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    if shared.view.get().players.get() == 1 && shared.view_car(0).is_none() {
        return Err(crate::errors::BadInput("a null view block").into());
    }
    run(shared.car(), numbers);
    let speed = x87::loaded(&shared.car().engine_speed);
    let floor = shared.physics_tables.get().idle_engine_speed.get();
    let held = hold_low(speed, floor);
    let whole = ext_to_int(extended(held.to_bits())) as u32;
    let (revs, _) = ease_revs(shared.car().engine_sound_revs.get(), whole);
    shared.car().engine_sound_revs.set(revs);
    { car_sound_state(numbers, matrices, shared, sound) }?;
    gear_change_sound(shared.car(), sound, shared.work.get())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
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
            plain: |_| {
                run_at(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        }];
        sparse("physics.engine_sound", &blocks, 4000, 200);
    }
}
