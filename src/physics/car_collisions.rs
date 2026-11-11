//! `physics::car_collisions` (0x45F615): the work car against the other cars, in race modes 0
//! and 2 (`run_kind`; else nothing). For each car n below `g_raceCars` but the current one
//! (`g_currentCar`) whose block (`g_carBlocks[n]`, kept in `other_car_block`, n in
//! `other_car_number`) has `driver_kind` set: the difference of the places becomes the place
//! (`scene::set_place`), `g_crashMatrix` times that goes into `g_turnedMatrix`, and that times
//! the block's `car_inverse_matrix` goes into `corner_matrix` (`scene::multiply_matrices`); then
//! each of its eight corners (`corner_x`, `corner_y`, `corner_z`, into `corner_in`) goes through
//! `physics::corner_hits_car`; `corner_inside` starts at 0.
//!
//! Returns: the cars' count (at least 1) in modes 0 and 2
//!
//! Callers: `physics::car_physics_step` (every physics step), `physics::find_clear_place`. Refuses
//! (before anything) more than eight cars or a null block among them (the original reads past
//! `g_carBlocks` or through null), and what `corner_hits_car` refuses (what was written put back).

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::corner_hits_car::corner_hits_car;
use crate::physics::step_inputs::StepInputs;
use crate::race::view_store::RaceView;
use crate::scene::matrix::{multiply_matrices, set_place};

/// The cars looked at: `g_raceCars`, at least one (the loop runs once).
fn cars(view: &RaceView) -> u32 {
    view.race_cars.get().max(1)
}

pub fn car_collisions(
    c: &StepInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let physics = c.physics;
    if !matches!(c.vars.run_kind.get(), 0 | 2) {
        return Ok(());
    }
    let n = cars(c.view);
    if n > 8 {
        return Err(BadInput("more than eight cars (g_carBlocks' room)").into());
    }
    if (0..n).any(|k| k != c.work.current_car.get() && c.work.car_block(k as usize) == 0) {
        return Err(BadInput("a null car block").into());
    }
    {
        physics.contact.corner_inside.set(0);
        for car in 0..n {
            if car == c.work.current_car.get() {
                continue;
            }
            physics.contact.other_car_number.set(car);
            let block = c.work.car_block(car as usize);
            physics.contact.other_car_block.set(block);
            // SAFETY: a `g_carBlocks` entry of the eight, not null (both checked above).
            if { Car::at(block) }.driver_kind.bits() == 0 {
                continue;
            }
            let place = [0usize, 1, 2].map(|k| {
                c.car.position[k]
                    .get()
                    .wrapping_sub(Car::at(block).position[k].get())
            });
            set_place(&c.frame.matrices.placed_matrix, place);
            // `scene::set_place` left the z in `g_scratch`.
            crate::scene::owner::scene()
                .drawing
                .scene
                .scratch
                .set_bits(place[2]);
            multiply_matrices(
                &c.car.crash_matrix,
                &c.frame.matrices.placed_matrix,
                &c.frame.matrices.turned_matrix,
            );
            multiply_matrices(
                &c.frame.matrices.turned_matrix,
                &Car::at(block).car_inverse_matrix,
                &physics.contact.corner_matrix,
            );
            for k in 0..8 {
                physics.contact.corner_in[0].set_bits(physics.contact.corner_x[k].bits());
                physics.contact.corner_in[1].set_bits(physics.contact.corner_y[k].bits());
                physics.contact.corner_in[2].set_bits(physics.contact.corner_z[k].bits());
                { corner_hits_car(&c.contact(), block, scene) }?;
            }
        }
        Ok(())
    }
}
