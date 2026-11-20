//! The ground under each of the car's `probe_count` probe points (the work car's lists
//! `probe_point_x`, `probe_point_y`, `probe_point_z`, 0x40 each; the first four are the wheels'
//! points: they alone get their y replaced below, and they feed `wheel_ground_x / height / z` and
//! `wheel_lifted`). Nothing when the count is 0. Otherwise `crossed_probe` = -1 and for each point
//! b:
//! - for b < 4 its y is replaced: the list's seventh y less `handling[4]`, plus `probe_lift` unless
//!   bit b + 5 of `parts_present` is set and then unless bit b + 1 is; less `probe_share` times
//!   `handling[3]` (the point's own y only goes to the scratch dword);
//! - the point is turned by the crash matrix (`scene::rotate_point`), its x y z kept as floats
//!   (`probe_turned_x`, `probe_turned_y`, `probe_turned_z`; x and z also in `probe_turned_x_last`
//!   and `probe_turned_z_last`), and as whole numbers times 65536 plus the car's `position` in
//!   `probe_x`, `probe_y`, `probe_z`;
//! - the ground under (x, z) comes from `physics::find_ground`, with the mask `settle_mark` 0x80
//!   and the last face `probe_point_face[b]`; `probe_found[b]` records the result;
//! - not found: `probe_ground_height[b]` = 0, `crossed_probe` = b;
//! - found: `probe_ground_height[b]` is the height (`ground_height`) plus a bump: the noise bytes
//!   (`noise_bytes`, 64 x 64) at (x >> 14, z >> 14) mixed by the low 14 bits
//!   (`ground_noise_corners` the steps), times `ground_bump_scale`, 256, over 100; the face's half
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
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::math::x87::less_plus_less_product_80;
use crate::math::x87_float as bh;
use crate::memory::session::{CellKind, DwordLike, FloatLike};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::ground::*;
use crate::physics::ground_query::GroundQuery;
use crate::physics::step_families::GroundInputs;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::ground;

pub const MOST_POINTS: u32 = 0x40;

/// What the probe reads besides the car and the query: the drawing parts build it from the
/// race's shared parts (the work car and the physics' ground work are the game thread's).
#[derive(Clone, Copy)]
pub struct ProbeInputs<'a> {
    pub numbers: &'a crate::race::numbers_store::Numbers,
    pub tables: &'a crate::physics::tables_store::PhysicsTables,
    pub course: crate::race::owner_course::CourseReads<'a>,
    pub engine_tables: &'a crate::math::engine_tables::EngineTables,
}

impl<'a> ProbeInputs<'a> {
    pub fn of_ground(c: &GroundInputs<'a>) -> Self {
        ProbeInputs {
            numbers: c.numbers,
            tables: c.tables,
            course: c.course.reads(),
            engine_tables: c.engine_tables,
        }
    }
}

/// The steps.
// Core: `rc2k_core::physics::ground::{wheel_lifted, noise_cell, bump_noise}` (the turns and the
// bump's scaling stay x87).
pub fn probe_ground_at(
    c: &ProbeInputs<'_>,
    car: &Car<impl CellKind>,
    q: &GroundQuery<'_>,
    contact: &ContactWork,
) -> Result<(), Refused> {
    let n = car.probe_count.get();
    if n < 1 {
        return Ok(());
    }
    if n > MOST_POINTS {
        return Err(BadInput("more than 0x40 probe points (the lists' room)").into());
    }

    probe_points(c, car, n, q, contact)
}

fn probe_points(
    c: &ProbeInputs<'_>,
    car: &Car<impl CellKind>,
    n: u32,
    q: &GroundQuery<'_>,
    contact: &ContactWork,
) -> Result<(), Refused> {
    // The work values the point's steps spill through (each is stored before it is read).
    let scratch = crate::memory::session::PlainDword::new(0);
    let k65536 = c.numbers.k65536.get();
    contact.crossed_probe.set(u32::MAX);
    for b in 0..n {
        let i = b as usize;
        let work = contact;
        let z = car.probe_point_z[i].get();
        let y = car.probe_point_y[i].get();
        let y = if b < 4 {
            // The point's height goes through the scratch dword (a float's bits).
            scratch.set(x87::loaded(y).to_bits());
            let add;
            (add, _) = ground::wheel_lifted(car.parts_present.get(), b);
            less_plus_less_product_80(
                &car.probe_point_y[6],
                &c.tables.handling[2],
                add.then(|| c.tables.probe_lift.get()),
                &c.tables.probe_share,
                &c.tables.handling[3],
            )
        } else {
            extended(y)
        };
        let x = extended(car.probe_point_x[b as usize].get());
        let out = rotate_point(&car.crash_matrix, [x, y, extended(z)]);
        let [ox, oy, oz] = out.map(rounded_value);
        contact.probe_turned_x_last.set(ox);
        work.probe_turned_x[i].set(ox);
        work.probe_turned_y[i].set(oy);
        contact.probe_turned_z_last.set(oz);
        work.probe_turned_z[i].set(oz);
        for (v, place, list) in [
            (ox, &car.position[0], &work.probe_x[i]),
            (oy, &car.position[1], &work.probe_y[i]),
            (oz, &car.position[2], &work.probe_z[i]),
        ] {
            let w = (x87::mul_to_int(v, k65536) as u32).wrapping_add(place.get());
            list.set(w);
        }
        q.x.set(work.probe_x[i].get());
        q.z.set(work.probe_z[i].get());
        // The search tries this face first (`ground_face_last`: the physics' query's own cell,
        // a drawing's its own).
        q.last.set(car.probe_point_face[b as usize].get());
        q.settle_mark.set(0x80);
        ground_search_at(c.course, q)?;
        work.probe_ground_height[i].set(0);
        if q.found.get() == 0 {
            contact.crossed_probe.set(b);
            work.probe_found[i].set(0);
            continue;
        }
        work.probe_found[i].set(1);
        work.probe_ground_height[i].set(q.height.get());
        let cell = ground::noise_cell(q.x.get(), q.z.get());
        contact.ground_cell_row[0].set(cell.row);
        contact.ground_cell_row[1].set(cell.row_offset);
        let trap = crate::physics::cells::Trap::new();
        let noise = |i: u32| {
            trap.keep(
                c.engine_tables
                    .noise_bytes
                    .byte_at(i as usize, "the ground noise table")
                    .map(u32::from),
            )
        };
        let n = cell.corners.map(noise);
        trap.finish()?;
        for (c, x) in [0, 1, 3, 2].into_iter().zip(n) {
            work.ground_noise_corners[c].set(x);
        }
        let (n58, n54, n5c, _) = ground::bump_noise(n, q.x.get(), q.z.get());
        contact.ground_noise_mix[1].set(n58);
        contact.ground_noise_mix[0].set(n54);
        contact.ground_noise_mix[2].set(n5c);
        scratch.set(n5c);
        let bump = bh::to_int(
            bh::int(n5c as i32) * car.ground_bump_scale.ld() * c.numbers.k256.ld()
                / c.numbers.hundred.ld(),
        );
        scratch.set(bump as u32);
        work.probe_ground_height[i]
            .set(work.probe_ground_height[i].get().wrapping_add(bump as u32));
        work.probe_face_flip[i].set(q.half.get());
        work.probe_share_u[i].set(q.plane_f.get());
        work.probe_share_v[i].set(q.plane_g.get());
        let d = work.probe_y[i]
            .get()
            .wrapping_sub(work.probe_ground_height[i].get());
        work.probe_height[i].set(d);
        if b < 4 {
            scratch.set(x87::loaded(&car.crash_matrix[5]).to_bits());
            if x87::compare_status(scratch.get(), ZERO) & 0x100 != 0 {
                scratch.set(x87::loaded(ZERO).to_bits());
            }
            work.wheel_height_raw[i].set(d);
            work.probe_height[i].set(x87::int_times_to_int(d as i32, scratch.get()) as u32);
        }
        car.probe_point_face[b as usize].set(q.slot.get());
    }
    for i in 0..4 {
        car.wheel_ground_x[i].set(contact.probe_x[i].get());
        car.wheel_ground_height[i].set(contact.probe_ground_height[i].get());
        car.wheel_ground_z[i].set(contact.probe_z[i].get());
    }
    Ok(())
}

/// `car` is the work car in the race's ticks (the view's copy of its car in the drawing,
/// `probe_ground_in_view`).
pub fn probe_ground(c: &GroundInputs<'_>, car: &Car) -> Result<(), Refused> {
    probe_ground_at(
        &ProbeInputs::of_ground(c),
        car,
        &GroundQuery::of_physics(
            c.ground,
            c.probe,
            c.course.course.get(),
            &crate::race::scene_points::no_own(),
        ),
        c.contact,
    )
}

/// The same for the drawing (`race::face_hide_mask`): its ground search, its probe lists and its
/// work values on fields of its own, not the physics' work and the scene's scratch (nothing the
/// drawing does reads them after), the scene points through the drawing part's `points`.
pub fn probe_ground_in_view(
    c: &ProbeInputs<'_>,
    car: &Car<impl CellKind>,
    points: &crate::race::scene_points::OwnPoints,
) -> Result<(), Refused> {
    let ground = crate::physics::ground_query::GroundCells::new();
    probe_ground_at(c, car, &ground.query(points), &ContactWork::new())
}
