//! `physics::set_wheel_weights` (0x43F3E7): the four wheels' load forces (the records at
//! 0x614770 + 0x20 k, `physics::clear_forces`'): a share `flt_6D3070` held to 1 ..
//! `flt_5C5D48` (through `dword_62400C`), ((it - 1) `flt_5C5DA4` + 1) gravity
//! (`flt_613FC8`) `flt_614170`, negated, times `flt_5C5DA4` again into each record's +4,
//! turned into the car's frame (`physics::turn_by_inverse`); then each record's second
//! vector (+0xC ..) set to (-1 / 1, 0, ±1) by its corner and turned the same way. A
//! reading: the weight on each wheel, the corners' places.
//!
//! In:  nothing
//! Out: esi 0x6147DC (the last turned); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B723), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87;
use crate::math::x87_run::{compare_full, weight_force};
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;

/// The Rust version. The registers the original leaves are read by none of its callers (esi).
pub fn set_wheel_weights() {
    let scratch = &DRAW.scratch;
    let one: u32 = NUMBERS.one.bits();
    let high: u32 = NUMBERS.five.bits();
    let k: u32 = NUMBERS.quarter.bits();
    scratch.set(x87::loaded(CONTACT_WORK.wheel_share.bits()));
    if compare_full(scratch.get(), one) & 0x100 != 0 {
        scratch.set(x87::loaded(one));
    }
    if compare_full(scratch.get(), high) & 0x100 == 0 {
        scratch.set(x87::loaded(high));
    }
    let f = weight_force(
        scratch.get(),
        one,
        k,
        PHYSICS.gravity.bits(),
        PHYSICS.weight.bits(),
    );
    let wheels = &PHYSICS.forces[..4];
    for w in wheels {
        w.force[1].set_bits(f);
    }
    for w in wheels {
        turn_by_inverse(&w.force);
    }
    let plus = x87::loaded(1.0f32.to_bits());
    let minus = x87::negated(plus);
    // The wheels' points: x and z of +1 or -1 (0x2C, 0x6C, 0x14, 0x34 plus; 0x0C, 0x4C,
    // 0x54, 0x74 minus).
    for cell in [
        &wheels[1].point[0],
        &wheels[3].point[0],
        &wheels[0].point[2],
        &wheels[1].point[2],
    ] {
        cell.set_bits(plus);
    }
    for cell in [
        &wheels[0].point[0],
        &wheels[2].point[0],
        &wheels[2].point[2],
        &wheels[3].point[2],
    ] {
        cell.set_bits(minus);
    }
    for w in wheels {
        turn_by_inverse(&w.point);
    }
}
