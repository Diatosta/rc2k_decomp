//! `physics::wheel_springs` (0x43ECD4): the four wheels' springs from their heights. For each
//! wheel k (0 .. 3):
//! - the last squeeze (`wheel_squeeze[k]`) is kept in `wheel_squeeze_last[k]` and the scene's
//!   work dword;
//! - the new squeeze = -`probe_height[k]` (the wheel's height, as a whole number) over 65536
//!   and the handling table's scale (`handling[3]`), held to 0 ..= 1;
//! - its force `spring_forces[k]` = -(squeeze x `spring_force_scale` x the first force record's
//!   y); the damping `spring_damping[k]` = (squeeze - last) x `handling[5]`, times `handling[0]`
//!   when below 0 (or unordered), added to the force unless the squeeze is 0 (or unordered).
//!
//! Then `wheel_spring_force[k]` = the rounded result of the ride-height sum (`handling[2]` -
//! the first car's probe point y, + `spring_share`, + (squeeze - `probe_share`) x `handling[3]`)
//! x 65536; the forces go into the y of the wheel force records 6 .. 9; those records' points
//! come from the wheels' places (`probe_point_x`, `probe_point_y`, `probe_point_z`), copied on
//! into the records 11 .. 14 and 15 .. 18; the point y of record 10 is 0; minus the
//! `wheel_spin_push` figures go into the y of records 11 .. 14 and 15 .. 18.
//!
//! Caller: `physics::car_physics_step`, every physics step.

use crate::math::x87;
use crate::math::x87::{ext_div2, int_80, mul2, neg_rounded, negated, wheel_int};
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::blocks_store::CarBlocks;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix3::extended;
use std::cmp::Ordering;

/// `fld b; fld x; fcompp` (or `fld x; fcomp b`), C0: x below b, or unordered.
fn below(x: f32, b: f32) -> bool {
    !matches!(x.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

/// The exact version, for `physics::car_physics_step`.
pub fn wheel_springs(
    car_blocks: &CarBlocks,
    contact: &ContactWork,
    numbers: &Numbers,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    let (c, ph) = (shared.car(), &shared.physics_tables);
    for k in 0..4 {
        let q = &c.wheel_squeeze[k];
        let (force, damping) = (&ph.spring_forces[k], &ph.spring_damping[k]);
        let old = x87::loaded(q);
        drawing.scene.work_dword.set(old.to_bits());
        c.wheel_squeeze_last[k].set(old);
        let eax = contact.probe_height[k].get().wrapping_neg();
        drawing.scene.scratch.set(eax);
        q.set(ext_div2(
            int_80(eax as i32, false),
            &numbers.k65536,
            &ph.handling[3],
        ));
        if below(q.get(), 0.0) {
            q.set(x87::loaded(ZERO));
        }
        if !below(q.get(), numbers.one.get()) {
            q.set(x87::loaded(&numbers.one));
        }
        force.set(negated(mul2(
            q,
            &ph.spring_force_scale,
            &ph.forces[0].force[1],
        )));
        damping.set(x87::sub_mul(
            q,
            drawing.scene.work_dword.get(),
            ph.handling[5].bits(),
        ));
        if below(damping.get(), 0.0) {
            damping.set(x87::mul(damping, &ph.handling[0]));
        }
        let zero_or_none = q
            .get()
            .partial_cmp(&0.0)
            .is_none_or(|o| o == Ordering::Equal);
        if !zero_or_none {
            force.set(x87::add(damping, force));
        }
    }
    for k in 0..4 {
        let v = wheel_int([
            car_blocks.cars[0].probe_point_y[6].bits(),
            ph.handling[2].bits(),
            ph.spring_share.bits(),
            c.wheel_squeeze[k].bits(),
            ph.probe_share.bits(),
            ph.handling[3].bits(),
            numbers.k65536.bits(),
        ]);
        c.wheel_spring_force[k].set(v as u32);
    }
    // The springs' records (6 .. 9): the force up, the wheels' places.
    for k in 0..4 {
        ph.forces[6 + k].force[1].set(x87::loaded(&ph.spring_forces[k]));
    }
    for (row, from) in [&c.probe_point_x, &c.probe_point_y, &c.probe_point_z]
        .into_iter()
        .enumerate()
    {
        for (record, place) in ph.forces[6..10].iter().zip(from) {
            record.point[row].set(x87::loaded(place));
        }
    }
    // Their x and z copied on into the records 11 .. 14 and 15 .. 18, in the original's
    // order.
    for k in 0..4 {
        for row in [0, 2] {
            let v = x87::loaded(&ph.forces[6 + k].point[row]);
            ph.forces[11 + k].point[row].set(v);
            ph.forces[15 + k].point[row].set(v);
        }
    }
    ph.forces[10].point[1].set(0.0);
    let n = neg_rounded(extended(ph.wheel_spin_push[0].get()));
    for k in 0..4 {
        ph.forces[11 + k].point[1].set(n);
    }
    let n = neg_rounded(extended(ph.wheel_spin_push[1].get()));
    for k in 0..4 {
        ph.forces[15 + k].point[1].set(n);
    }
}
