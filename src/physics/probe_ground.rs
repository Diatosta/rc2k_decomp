//! The ground under each of the car's `probe_count` probe points (the work car's lists
//! `probe_point_x`, `probe_point_y`, `probe_point_z`, 0x40 each; the first four are the wheels'
//! points: they alone get their y replaced below, and they feed `wheel_ground_x / height / z` and
//! `wheel_lifted`). Nothing when the count is 0. Otherwise `crossed_probe` = -1 and for each point
//! b:
//! - for b < 4 its y is replaced: the list's seventh y less `handling[4]`, plus `probe_lift` unless
//!   bit b + 5 of `parts_present` is set and then unless bit b + 1 is; less `probe_share` times
//!   `handling[3]` (the point's own y only goes to the scratch dword);
//! - the point is turned by the crash matrix (`scene::rotate_point`), its x y z kept as floats
//!   (`probe_turned_x`, `probe_turned_y`, `probe_turned_z`), and as whole numbers times 65536 plus the car's `position` in
//!   `probe_x`, `probe_y`, `probe_z`;
//! - the ground under (x, z) comes from `physics::find_ground`, with the mask `settle_mark` 0x80
//!   and the last face `probe_point_face[b]`; `probe_found[b]` records the result;
//! - not found: `probe_ground_height[b]` = 0, `crossed_probe` = b;
//! - found: `probe_ground_height[b]` is the height (`ground_height`) plus a bump: the noise bytes
//!   (`noise_bytes`, 64 x 64) at (x >> 14, z >> 14) mixed by the low 14 bits, times `ground_bump_scale`, 256, over 100; the face's half
//!   and shares (`probe_face_flip`, `probe_share_u`, `probe_share_v`); `probe_height[b]` = y less
//!   that ground; for b < 4 that also goes into `wheel_height_raw[b]` and `probe_height[b]` is
//!   multiplied by the car's wheel-load figure (held to at least 0); `probe_point_face[b]` = the
//!   face found.
//!
//! Then the first four of `probe_x`, `probe_ground_height`, `probe_z` go into the car's
//! `wheel_ground_x`, `wheel_ground_height`, `wheel_ground_z`.
//!
//! Refuses (before anything: what it wrote is put back) a count above 0x40 (the lists' room), and
//! what `find_ground` refuses on any point.

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::PlainX87;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::math::x87::less_plus_less_product_80;
use crate::math::x87_float as bh;
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ProbeLists;
use crate::physics::figures::{PROBE_LIFT, PROBE_SHARE};
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::ground::*;
use crate::physics::ground_query::{GroundEnv, GroundSearch};
use crate::physics::step_families::GroundInputs;
use crate::race::scene_points::OwnPoints;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::ground;

pub const MOST_POINTS: u32 = 0x40;

/// The mark the probe points' searches are made with (which faces count).
const SETTLE_MARK: u32 = 0x80;

/// What the probe reads besides the car and the query: the drawing parts build it from the
/// race's shared parts (the work car and the physics' ground work are the game thread's).
#[derive(Clone, Copy)]
pub struct ProbeInputs<'a> {
    pub numbers: &'a crate::race::numbers_store::Numbers,
    /// `PhysicsTables.handling[2]` and `handling[3]`.
    pub handling: [f32; 2],
    pub course: crate::race::owner_course::CourseReads<'a>,
    pub engine_tables: crate::math::engine_tables::EngineTablesRef<'a>,
}

impl<'a> ProbeInputs<'a> {
    pub(super) fn of_ground(c: &GroundInputs<'a>) -> Self {
        ProbeInputs {
            numbers: c.numbers,
            handling: {
                let tables = c.tables.borrow();
                [tables.handling[2], tables.handling[3]]
            },
            course: c.course.reads(),
            engine_tables: c.engine_tables,
        }
    }
}

/// The steps.
// Core: `rc2k_core::physics::ground::{wheel_lifted, noise_cell, bump_noise}` (the turns and the
// bump's scaling stay x87).
pub(super) fn probe_ground_at(
    c: &ProbeInputs<'_>,
    car: &mut Car,
    q: &mut GroundSearch,
    points: &OwnPoints,
    work: &mut ProbeLists,
) -> Result<(), Refused> {
    let n = car.probe_count;
    if n < 1 {
        return Ok(());
    }
    if n > MOST_POINTS {
        return Err(BadInput("more than 0x40 probe points (the lists' room)").into());
    }

    probe_points(c, car, n, q, points, work)
}

fn probe_points(
    c: &ProbeInputs<'_>,
    car: &mut Car,
    n: u32,
    q: &mut GroundSearch,
    points: &OwnPoints,
    work: &mut ProbeLists,
) -> Result<(), Refused> {
    // The work values the point's steps spill through (each is stored before it is read).
    let scratch = crate::memory::session::PlainDword::new(0);
    let k65536 = c.numbers.k65536.get();
    work.crossed_probe = u32::MAX;
    for b in 0..n {
        let i = b as usize;
        let z = car.probe_point_z[i];
        let y = car.probe_point_y[i];
        let y = if b < 4 {
            // The point's height goes through the scratch dword (a float's bits).
            scratch.set(x87::loaded(y).to_bits());
            let add;
            (add, _) = ground::wheel_lifted(car.parts_present, b);
            less_plus_less_product_80(
                car.probe_point_y[6],
                c.handling[0],
                add.then_some(PROBE_LIFT),
                PROBE_SHARE,
                c.handling[1],
            )
        } else {
            extended(y)
        };
        let x = extended(car.probe_point_x[b as usize]);
        let out = rotate_point(&car.crash_matrix, [x, y, extended(z)]);
        let [ox, oy, oz] = out.map(rounded_value);
        work.probe_turned_x[i] = ox;
        work.probe_turned_y[i] = oy;
        work.probe_turned_z[i] = oz;
        for (v, axis, list) in [
            (ox, 0, &mut work.probe_x[i]),
            (oy, 1, &mut work.probe_y[i]),
            (oz, 2, &mut work.probe_z[i]),
        ] {
            let w = (x87::mul_to_int(v, k65536) as u32).wrapping_add(car.position.bits_at(axis));
            *list = w;
        }
        q.x = work.probe_x[i];
        q.z = work.probe_z[i];
        // The search tries this face first.
        q.last = car.probe_point_face[b as usize];
        let env = GroundEnv {
            course: c.course,
            points,
            mark: SETTLE_MARK,
        };
        ground_search_at(env, q)?;
        work.probe_ground_height[i] = 0;
        if q.found == 0 {
            work.crossed_probe = b;
            work.probe_found[i] = 0;
            continue;
        }
        work.probe_found[i] = 1;
        work.probe_ground_height[i] = q.height;
        let cell = ground::noise_cell(q.x, q.z);
        let trap = crate::physics::cells::Trap::new();
        let noise = |i: u32| {
            trap.keep(
                crate::math::engine_tables::noise_byte(
                    c.engine_tables,
                    i as usize,
                    "the ground noise table",
                )
                .map(u32::from),
            )
        };
        let n = cell.corners.map(noise);
        trap.finish()?;
        let (_, _, n5c, _) = ground::bump_noise(n, q.x, q.z);
        scratch.set(n5c);
        let bump = bh::to_int(
            bh::int(n5c as i32) * car.ground_bump_scale.ld() * c.numbers.k256.ld()
                / c.numbers.hundred.ld(),
        );
        scratch.set(bump as u32);
        work.probe_ground_height[i] = work.probe_ground_height[i].wrapping_add(bump as u32);
        work.probe_face_flip[i] = q.half;
        work.probe_share_u[i] = q.plane_f;
        work.probe_share_v[i] = q.plane_g;
        let d = work.probe_y[i].wrapping_sub(work.probe_ground_height[i]);
        work.probe_height[i] = d;
        if b < 4 {
            scratch.set(x87::loaded(car.crash_matrix.at(5)).to_bits());
            if x87::compare_status(scratch.get(), ZERO) & 0x100 != 0 {
                scratch.set(x87::loaded(ZERO).to_bits());
            }
            work.probe_height[i] = x87::int_times_to_int(d as i32, scratch.get()) as u32;
        }
        car.probe_point_face[b as usize] = q.slot;
    }
    for i in 0..4 {
        car.wheel_ground_x[i] = work.probe_x[i];
        car.wheel_ground_height[i] = work.probe_ground_height[i];
        car.wheel_ground_z[i] = work.probe_z[i];
    }
    Ok(())
}

/// `car` is the work car in the race's ticks (the view's copy of its car in the drawing,
/// `probe_ground_in_view`). The mark the searches were made with is kept in the probe, where
/// the drawing parts read it.
pub fn probe_ground(c: &GroundInputs<'_>, car: &mut Car) -> Result<(), Refused> {
    if (1..=MOST_POINTS).contains(&car.probe_count) {
        c.probe.borrow_mut().settle_mark = SETTLE_MARK;
    }
    probe_ground_at(
        &ProbeInputs::of_ground(c),
        car,
        &mut c.search.borrow_mut(),
        &crate::race::scene_points::no_own(),
        &mut c.probes.borrow_mut(),
    )
}

/// The same for the drawing (`race::face_hide_mask`): its ground search, its probe lists and its
/// work values on records of its own, not the physics' work and the scene's scratch (nothing the
/// drawing does reads them after), the scene points through the drawing part's `points`.
pub fn probe_ground_in_view(
    c: &ProbeInputs<'_>,
    car: &mut Car,
    points: &OwnPoints,
) -> Result<(), Refused> {
    probe_ground_at(
        c,
        car,
        &mut GroundSearch::default(),
        points,
        &mut ProbeLists::new(),
    )
}
