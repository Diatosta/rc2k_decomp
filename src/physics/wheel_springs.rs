//! `physics::wheel_springs` (0x43ECD4): the four wheels' springs from their heights. For
//! each wheel k (ebx 0 .. 3):
//! - the last squeeze `qword_71A5D8[k]` (g_car +0x600) kept in `dword_623FB8` and
//!   `flt_71A5E8[k]` (+0x610);
//! - the new squeeze = -`dword_6D2514[k]` (the wheel's height, through `dword_62400C`,
//!   `fild`) over 65536 (`flt_5C5D98`) and `flt_614BF8`, held to 0 ..= 1
//!   (`flt_5C5D90`, `flt_5C5D38`);
//! - its force `flt_614178[k]` = -(squeeze x `flt_613F90` x `flt_614774`); the damping
//!   `flt_614124[k]` = (squeeze - last) x `flt_614C00`, times `flt_614BEC` when below 0
//!   (or unordered), added to the force unless the squeeze is 0 (or unordered).
//!
//! Then `dword_71A8E8[k]` = `fistp` of ((`flt_70FFF4` - `flt_614BF4` + `flt_613EC0`) +
//! (squeeze - `flt_613F8C`) x `flt_614BF8`) x 65536; the forces into the wheel records'
//! +4 (`unk_614830` + 0x20 k); the records' +0xC .. +0x14 from the wheels' places
//! (`flt_71AAC8` .., `flt_71ABC8` .., `flt_71ACC8` ..), copied on into the records at
//! 0x6148DC .. and 0x61495C ..; `flt_6148C0` 0; minus `flt_6141D4` into 0x6148E0,
//! 0x614900, 0x614920, 0x614940 and minus `flt_6141D8` into 0x614960 .. 0x6149C0.
//!
//! In:  nothing
//! Out: ebx 4, eax minus wheel 3's height; the rest kept; no flags read (the next is a
//!      call)
//!
//! Caller: `sub_43B635` (0x43B813), every physics step.

use crate::math::x87;
use crate::math::x87_run::{ext_div2, int_80, neg_mul2, neg_rounded, sub_mul, wheel_int};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::extended;
use std::cmp::Ordering;

/// `fld b; fld x; fcompp` (or `fld x; fcomp b`), C0: x below b, or unordered.
fn below(x: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(x).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn wheel_springs() {
    let (c, ph) = (&CAR, &PHYSICS);
    for k in 0..4 {
        let q = &c.wheel_squeeze[k];
        let (force, damping) = (&ph.spring_forces[k], &ph.spring_damping[k]);
        let old = x87::loaded(q.bits());
        DRAW.work_dword.set(old);
        c.wheel_squeeze_last[k].set_bits(old);
        let eax = crate::race::owner::race().physics.contact.probe_height[k]
            .get()
            .wrapping_neg();
        DRAW.scratch.set(eax);
        q.set_bits(ext_div2(
            int_80(eax as i32, false),
            crate::race::owner::race().course.numbers.k65536.bits(),
            ph.handling[3].bits(),
        ));
        if below(
            q.bits(),
            crate::race::owner::race().course.numbers.zero.bits(),
        ) {
            q.set_bits(x87::loaded(
                crate::race::owner::race().course.numbers.zero.bits(),
            ));
        }
        if !below(
            q.bits(),
            crate::race::owner::race().course.numbers.one.bits(),
        ) {
            q.set_bits(x87::loaded(
                crate::race::owner::race().course.numbers.one.bits(),
            ));
        }
        force.set_bits(neg_mul2(
            q.bits(),
            ph.spring_force_scale.bits(),
            ph.forces[0].force[1].bits(),
        ));
        damping.set_bits(sub_mul(
            q.bits(),
            DRAW.work_dword.get(),
            ph.handling[5].bits(),
        ));
        if below(
            damping.bits(),
            crate::race::owner::race().course.numbers.zero.bits(),
        ) {
            damping.set_bits(x87::mul(damping.bits(), ph.handling[0].bits()));
        }
        let zero_or_none = f32::from_bits(q.bits())
            .partial_cmp(&f32::from_bits(
                crate::race::owner::race().course.numbers.zero.bits(),
            ))
            .is_none_or(|o| o == Ordering::Equal);
        if !zero_or_none {
            force.set_bits(x87::add(damping.bits(), force.bits()));
        }
    }
    for k in 0..4 {
        let v = wheel_int([
            CAR_BLOCKS.cars[0].probe_point_y[6].bits(),
            ph.handling[2].bits(),
            ph.spring_share.bits(),
            c.wheel_squeeze[k].bits(),
            ph.probe_share.bits(),
            ph.handling[3].bits(),
            crate::race::owner::race().course.numbers.k65536.bits(),
        ]);
        c.wheel_spring_force[k].set(v as u32);
    }
    // The springs' records (6 .. 9): the force up, the wheels' places.
    for k in 0..4 {
        ph.forces[6 + k].force[1].set_bits(x87::loaded(ph.spring_forces[k].bits()));
    }
    for (row, from) in [&c.probe_point_x, &c.probe_point_y, &c.probe_point_z]
        .into_iter()
        .enumerate()
    {
        for (record, place) in ph.forces[6..10].iter().zip(from) {
            record.point[row].set_bits(x87::loaded(place.bits()));
        }
    }
    // Their x and z copied on into the records 11 .. 14 and 15 .. 18, in the original's
    // order.
    for k in 0..4 {
        for row in [0, 2] {
            let v = x87::loaded(ph.forces[6 + k].point[row].bits());
            ph.forces[11 + k].point[row].set_bits(v);
            ph.forces[15 + k].point[row].set_bits(v);
        }
    }
    ph.forces[10].point[1].set_bits(0);
    let n = neg_rounded(extended(ph.wheel_spin_push[0].bits()));
    for k in 0..4 {
        ph.forces[11 + k].point[1].set_bits(n);
    }
    let n = neg_rounded(extended(ph.wheel_spin_push[1].bits()));
    for k in 0..4 {
        ph.forces[15 + k].point[1].set_bits(n);
    }
}
