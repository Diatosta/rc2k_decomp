//! `physics::car_collisions` (0x45F615): the work car against the other cars, in race
//! modes 0 and 2 (`dword_62389C`; else nothing). `dword_6D3BB4` = 0; for each car n
//! below `g_raceCars` but the current one (`g_currentCar`), whose block
//! (`g_carBlocks[n]`, into `dword_6D3BB8`, n into `dword_6D3CD4`) has +0x25C set: the
//! places' difference as the place (`scene::set_place`), `g_crashMatrix` x that into
//! `g_turnedMatrix`, that x the block's matrix (+0x9CC) into `flt_6D3C40`
//! (`scene::multiply_matrices`); then for its eight corners (`dword_6D3BE0[k]`,
//! `dword_6D3C00[k]`, `dword_6D3C20[k]` into `dword_6D3BD4` ..) `physics::corner_hits_car`.
//! Every step in `pusha` / `popa`: only ebx, the count, is left.
//!
//! In:  nothing
//! Out: ebx the cars' count (at least 1) in modes 0 and 2; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. Refuses (before anything) more
//! than eight cars or a null block among them (the original reads past `g_carBlocks` or
//! through null), and what `corner_hits_car` refuses (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::corner_hits_car::corner_hits_car;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::scene::matrix::{multiply_matrices, set_place};

/// The cars looked at: `g_raceCars`, at least one (the loop runs once).
fn cars(shared: &Shared) -> u32 {
    shared.view.race_cars.get().max(1)
}

/// The Rust version. The registers it leaves (ebx the cars looked at) are read by none
/// of its callers.
pub fn car_collisions(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics = &race.physics;
    if !matches!(race.shared.vars.run_kind.get(), 0 | 2) {
        return Ok(());
    }
    let n = cars(&race.shared);
    if n > 8 {
        return Err(BadInput("more than eight cars (g_carBlocks' room)").into());
    }
    if (0..n).any(|k| {
        k != race.shared.work.current_car.get() && race.shared.work.car_block(k as usize) == 0
    }) {
        return Err(BadInput("a null car block").into());
    }
    {
        physics.contact.corner_inside.set(0);
        for car in 0..n {
            if car == race.shared.work.current_car.get() {
                continue;
            }
            physics.contact.other_car_number.set(car);
            let block = race.shared.work.car_block(car as usize);
            physics.contact.other_car_block.set(block);
            // SAFETY: a `g_carBlocks` entry of the eight, not null (both checked above).
            if { Car::at(block) }.driver_kind.bits() == 0 {
                continue;
            }
            set_place(
                &race.frame.matrices,
                [0usize, 1, 2].map(|k| {
                    race.shared.car.position[k]
                        .get()
                        .wrapping_sub(Car::at(block).position[k].get())
                }),
            );
            multiply_matrices(
                &race.shared.car.crash_matrix,
                &race.frame.matrices.placed_matrix,
                &race.frame.matrices.turned_matrix,
            );
            multiply_matrices(
                &race.frame.matrices.turned_matrix,
                &Car::at(block).car_inverse_matrix,
                &physics.contact.corner_matrix,
            );
            for k in 0..8 {
                physics.contact.corner_in[0].set_bits(physics.contact.corner_x[k].bits());
                physics.contact.corner_in[1].set_bits(physics.contact.corner_y[k].bits());
                physics.contact.corner_in[2].set_bits(physics.contact.corner_z[k].bits());
                { corner_hits_car(race, block, scene) }?;
            }
        }
        Ok(())
    }
}
