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
use crate::physics::corner_hits_car::corner_hits_car;
use crate::physics::step_families::CollisionInputs;
use crate::race::view_store::RaceView;
use crate::scene::matrix::{multiply_matrices, set_place};

/// The cars looked at: `g_raceCars`, at least one (the loop runs once).
fn cars(view: &RaceView) -> u32 {
    view.race_cars.get().max(1)
}

pub fn car_collisions(
    c: &CollisionInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (physics, work, car_now) = (c.contact.contact, c.contact.work, c.contact.car);
    if !matches!(c.vars.run_kind.get(), 0 | 2) {
        return Ok(());
    }
    let n = cars(c.view);
    if n > 8 {
        return Err(BadInput("more than eight cars (g_carBlocks' room)").into());
    }
    if (0..n).any(|k| k != work.current_car.get() && !work.has_car_block(k as usize)) {
        return Err(BadInput("a null car block").into());
    }
    {
        physics.corner_inside.set(0);
        for car in 0..n {
            if car == work.current_car.get() {
                continue;
            }
            physics.other_car_number.set(car);
            physics.other_car_block.set(work.car_handle(car as usize));
            // A `g_carBlocks` entry of the eight, not null (both checked above).
            let block = c.contact.shared.race_car_or_end(car as usize);
            if block.driver_kind.get() == 0 {
                continue;
            }
            let place = [0usize, 1, 2].map(|k| {
                car_now.position[k]
                    .get()
                    .wrapping_sub(block.position[k].get())
            });
            set_place(&c.matrices.placed_matrix, place);
            // `scene::set_place` left the z in `g_scratch`.
            scene.drawing.scene.get().scratch.set(place[2]);
            multiply_matrices(
                &car_now.crash_matrix,
                &c.matrices.placed_matrix,
                &c.matrices.turned_matrix,
            );
            multiply_matrices(
                &c.matrices.turned_matrix,
                &block.car_inverse_matrix,
                &physics.corner_matrix,
            );
            for k in 0..8 {
                physics.corner_in[0].set(physics.corner_x[k].get());
                physics.corner_in[1].set(physics.corner_y[k].get());
                physics.corner_in[2].set(physics.corner_z[k].get());
                { corner_hits_car(&c.contact, block, scene) }?;
            }
        }
        Ok(())
    }
}
