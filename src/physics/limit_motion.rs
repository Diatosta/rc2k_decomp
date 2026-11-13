//! `physics::limit_motion` (0x43F7AC): the work car's motion held to its limits. A motion
//! part that is no number or infinite (its exponent all ones) logs "*** INFINITY",
//! "[car0.newxspd] =" and the three (5 decimals) and zeroes the motion (+0x198), its car
//! frame copy (+0x1A4) and the spin (+0x184). Then the spin's parts held to
//! ±`spin_limit` and the motion's (x, z, y) to ±`velocity_limit` (the limit negated in place
//! for the second compare and put back); its length (`physics::vector_length`) into
//! `motion_length`, and past `top_speed` the motion and its copy scaled down to it
//! (`physics::scale_vector`, the factor then in `scratch`).
//!
//! The x87 compares are the arithmetic of the original's instructions over the game's
//! variables (from IDA), in plain Rust over `X` (`math/x87_float.rs`); the instructions are
//! recorded in the tests' `reference_vectors`.
//!
//! Callers: `physics::car_physics_step` (twice), `physics::find_clear_place`.

use crate::effects::cell_x87::CellX87;
use crate::logfile::log_float::log_float;
use crate::math::x87_float::compare;
use crate::physics::car_store::Car;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{rounded, rounded_value};
use crate::session::Cell32;
use std::cmp::Ordering;

/// The value at `v` held to ±(the limit at `l`), as the original: `fld l; fld v;
/// fcompp`, the limit negated in place for the second compare and put back.
fn hold(v: &impl Cell32, l: &impl Cell32) {
    let limit = l.ld();
    if compare(v.ld(), limit) & 0x100 == 0 {
        v.st(limit);
    }
    l.st(-l.ld());
    let negated = l.ld();
    if compare(v.ld(), negated) & 0x100 != 0 {
        v.st(negated);
    }
    l.st(-l.ld());
}

/// The exact version, for `physics::car_physics_step` and `physics::find_clear_place`.
pub fn limit_motion(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let mut bad = false;
    for item in car.velocity.iter().take(3usize) {
        // An integer test on the float's exponent bits (infinity or not a number).
        let ecx = item.bits() & 0x7F80_0000;
        if ecx == 0x7F80_0000 {
            bad = true;
            break;
        }
    }
    if bad {
        crate::logfile::log_write::log_nul(&c"*** INFINITY", false, 1);
        crate::logfile::log_write::log_nul(&c"[car0.newxspd] =", false, 1);
        for (k, cell) in car.velocity.iter().enumerate().take(3) {
            log_float(cell.get(), 5, k == 2, 1);
        }

        for k in 0..3usize {
            car.velocity[k].set(0.0);
            car.local_velocity[k].set(0.0);
            car.spin[k].set(0.0);
        }
    }
    for item in car.spin.iter().take(3usize) {
        hold(&item, &tables.spin_limit);
    }
    for k in [0usize, 2, 1] {
        hold(&car.velocity[k], &tables.velocity_limit);
    }
    let length = rounded_value(vector_length(&car.velocity));
    drawing.scene.get().motion_length.set(length);
    let top = tables.top_speed.get();
    let past =
        matches!(top.partial_cmp(&length), Some(Ordering::Less)) || top.is_nan() || length.is_nan();
    if past {
        let factor = crate::math::x87::div_80(top, length);
        scale_vector(&car.velocity, factor);
        scale_vector(&car.local_velocity, factor);
        drawing.scene.get().scratch.set(rounded(factor));
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    #[test]
    fn hold_matches_the_assembly() {
        let blocks = vec![Block {
            name: "limit_motion hold",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                hold(
                    &crate::scene::x87_mem::HostCell(a[0]),
                    &crate::scene::x87_mem::HostCell(a[1]),
                );
            },
        }];
        sparse("physics.limit_motion", &blocks, 6000, 201);
    }
}
