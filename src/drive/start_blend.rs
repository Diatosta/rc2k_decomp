//! `drive::start_blend` (0x43773D): the checkpoints the work car is past the lead's
//! start (`dword_71A070` - `dword_70F484`, held to 0 .. 100 by the signs), through
//! `dword_62400C` times `flt_604EC0` into `flt_71A654`; `flt_71A658` = `flt_71A650` +
//! that + `flt_71A64C`, held to `flt_5C5D90` .. `flt_5C5D38` (0 .. 1). By the look of
//! it a share used at a start (a reading).
//!
//! In:  nothing
//! Out: eax the held count; the rest kept; no flags read (`fld` next)
//!
//! Caller: `sub_43A35A` (0x43A486).

use crate::math::x87;
use crate::math::x87_run::{add3, compare_full};
use crate::physics::car_store::CAR;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner_course::Course;
use crate::race::tables_store::RACE_TABLES;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust core on the variables at `v`: the held count.
pub fn blend_at(course: &Course, v: &[&dyn Cell32]) -> u32 {
    let mut n = get(v, 0).wrapping_sub(get(v, 1));
    if (n as i32) < 0 {
        n = 0;
    }
    if (n.wrapping_sub(100) as i32) >= 0 {
        n = 100;
    }
    put(v, 2, n);
    put(
        v,
        3,
        x87::int_times(n as i32, RACE_TABLES.start_blend_step.bits()),
    );
    put(v, 6, add3(get(v, 4), get(v, 3), get(v, 5)));
    let (low, high) = (course.numbers.zero.bits(), course.numbers.one.bits());
    if compare_full(get(v, 6), low) & 0x100 != 0 {
        put(v, 6, x87::loaded(low));
    }
    if compare_full(get(v, 6), high) & 0x100 == 0 {
        put(v, 6, x87::loaded(high));
    }
    n
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.checkpoint,
        &CAR_BLOCKS.cars[0].checkpoint,
        &crate::scene::owner::scene().drawing.scene.scratch,
        &CAR.start_blend_share,
        &CAR.start_share,
        &CAR.pace_push,
        &CAR.start_blend,
    ]
}

/// The Rust version (the exact one, for `drive::read_car_controls`).
pub fn start_blend(course: &Course) -> u32 {
    blend_at(course, &game())
}
