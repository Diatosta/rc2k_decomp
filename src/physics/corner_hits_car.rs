//! Whether a corner of another car (`corner_in`, floats) lies inside the work car: the corner
//! times 65536 (`k65536`, unrounded) turned by `corner_matrix` and divided by 65536 into
//! `corner_turned`. Inside when each part's size is below the box's (`box_size`; unordered counts
//! as below): `corner_inside` = 1, then `physics::restore_saved_car` and `physics::bump_cars`.
//!
//! Takes: the other car's block (passed to `bump_cars`).
//!
//! Caller: `physics::car_collisions`, eight corners a car. Refuses what `restore_saved_car` and
//! `bump_cars` refuse (what was written is put back).

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87::{abs_compare, ext_times};
use crate::math::x87_float::from_f80;
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::ContactInputs;
use crate::race::numbers_store::Numbers;
use crate::scene::matrix3::extended;
use crate::scene::rotate_point::rotate_point;
use crate::session::Cell32;

/// Whether the corner is inside (`corner_turned` .. written either way).
pub fn corner_inside(contact: &ContactWork, numbers: &Numbers) -> bool {
    let k = numbers.k65536.get();
    let corner = contact
        .corner_in
        .each_ref()
        .map(|x| ext_times(extended(x.get()), k));
    let turned = rotate_point(&contact.corner_matrix, corner);
    for (n, e) in turned.iter().enumerate() {
        x87_1(numbers, e, &contact.corner_turned[n]);
    }
    (0..3).all(|n| abs_compare(&contact.corner_turned[n], &contact.box_size[n]) & 0x100 != 0)
}

/// `block` the other car's block.
pub fn corner_hits_car(
    c: &ContactInputs<'_>,
    block: &crate::physics::car_store::Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if !corner_inside(c.contact, c.numbers) {
        return Ok(());
    }
    c.contact.corner_inside.set(1);
    { crate::physics::restore_saved_car::restore_saved_car(c.car, c.tables, c.contact) }?;
    { crate::physics::bump_cars::bump_cars(c, block, scene) }?;
    Ok(())
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0] = ext [e + 0] / [k65536]`
fn x87_1(numbers: &Numbers, e: &[u8; 10], o: &impl Cell32) {
    let t0 = from_f80(*e);
    let t1 = numbers.k65536.ld();
    let t2 = t0 / t1;
    o.st(t2);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::{HostCell, view};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "corner_hits_car line 30",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::T)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    crate::test_support::own::race().course.numbers.get(),
                    view(a[0], 0),
                    &HostCell(a[2]),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.corner_hits_car", &blocks(), 1200, 100);
    }
}
