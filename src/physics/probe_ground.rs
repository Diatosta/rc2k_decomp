//! `physics::probe_ground` (0x45D305): the ground under each of the car's
//! `dword_71A238` probe points (the work copy's lists x `flt_71AAC8`, y `flt_71ABC8`, z
//! `flt_71ACC8`, 0x40 each; the first four by the look of it the wheels, a reading).
//! Nothing when the count is 0. Else `dword_6D3B84` = -1 and for each point b:
//! - for b < 4 its y is replaced: `flt_71ABE0` (the list's seventh y) less `flt_614BF4`,
//!   plus `flt_613E8C` unless bit b + 5 of `dword_71A270` is set and then unless bit
//!   b + 1 is; less `flt_613F8C` times `flt_614BF8` (the point's own y only goes to
//!   `dword_62400C`);
//! - the point turned by `g_crashMatrix` (`scene::rotate_point`), its x y z kept as floats
//!   (0x6D2214, 0x6D2314, 0x6D2414 [b]; x and z also in 0x6D3628 and 0x6D3630), and as
//!   whole numbers times 65536 plus the car's place (`dword_71A10C` ..) at 0x6D1E14,
//!   0x6D1F14, 0x6D2014 [b];
//! - the ground under (x, z) by `physics::find_ground`, the mask `dword_620470` 0x80, the
//!   last face `dword_71ADC8[b]`; `dword_6D2714[b]` the found;
//! - not found: `dword_6D2114[b]` 0, `dword_6D3B84` = b;
//! - found: `dword_6D2114[b]` the height (`dword_639828`) plus a bump: the noise bytes
//!   (`g_noiseBytes`, 64 x 64) at (x >> 14, z >> 14) mixed by the low 14 bits
//!   (0x6D3044 .. 0x6D3064 the steps), times `flt_71A038`, 256, over 100; the face's half
//!   and shares (0x6D2914, 0x6D2A14, 0x6D2B14 [b]); `dword_6D2514[b]` = y less that
//!   ground; for b < 4 that also into `dword_6D2614[b]` and `dword_6D2514[b]` times
//!   `flt_71A0A0` (held to at least `flt_5C5D90`); `dword_71ADC8[b]` = the face found.
//!
//! Then the first four of 0x6D1E14, 0x6D2114, 0x6D2014 into 0x71AEE8 .. 0x71AF14.
//!
//! Callers (6): `physics::find_clear_place`, `physics::feel_ground`, `race::settle_car` (twice),
//! `race::face_hide_mask`,
//! `physics::probe_and_keep`. Refuses (before anything: what it wrote is put back) a count above
//! 0x40 (the lists' room), and what `find_ground` refuses on any point.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{int_times_times_over_to_int, less_plus_less_product_80};
use crate::physics::car_store::Car;
use crate::physics::ground::*;
use crate::physics::ground_query::GroundQuery;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::ground;

pub const MOST_POINTS: u32 = 0x40;

/// The Rust core. The registers the original leaves (ebx the count, eax, ecx, edx, esi
/// the last point's) are read by none of its callers.
// Converted: `rc2k_core::physics::ground::{wheel_lifted, noise_cell, bump_noise}` (the
// turns and the bump's scaling stay x87).
/// `scratch` is the dword the original spills its work values to (`g_scratch`).
pub fn probe_ground_at(
    race: &Race,
    car: &Car,
    scene: &crate::scene::owner::Scene,
    q: &GroundQuery<'_>,
    scratch: &crate::session::Dword,
) -> Result<(), Refused> {
    let n = car.probe_count.get();
    if n < 1 {
        return Ok(());
    }
    if n > MOST_POINTS {
        return Err(BadInput("more than 0x40 probe points (the lists' room)").into());
    }

    probe_points(race, car, n, scene, q, scratch)
}

fn probe_points(
    race: &Race,
    car: &Car,
    n: u32,
    scene: &crate::scene::owner::Scene,
    q: &GroundQuery<'_>,
    scratch: &crate::session::Dword,
) -> Result<(), Refused> {
    let k65536 = race.course.numbers.k65536.bits();
    race.physics.contact.crossed_probe.set(u32::MAX);
    for b in 0..n {
        let i = b as usize;
        let work = &race.physics.contact;
        let z = car.probe_point_z[b as usize].bits();
        let y = car.probe_point_y[b as usize].bits();
        let y = if b < 4 {
            scratch.set(x87::loaded(y));
            let add;
            (add, _) = ground::wheel_lifted(car.parts_present.get(), b);
            less_plus_less_product_80(
                car.probe_point_y[6].bits(),
                race.shared.physics_tables.handling[2].bits(),
                add.then(|| race.shared.physics_tables.probe_lift.bits()),
                race.shared.physics_tables.probe_share.bits(),
                race.shared.physics_tables.handling[3].bits(),
            )
        } else {
            extended(y)
        };
        let x = extended(car.probe_point_x[b as usize].bits());
        let out = rotate_point(&car.crash_matrix, [x, y, extended(z)]);
        let (ox, oy, oz) = (rounded(out[0]), rounded(out[1]), rounded(out[2]));
        race.physics.contact.probe_turned_x_last.set_bits(ox);
        work.probe_turned_x[i].set_bits(ox);
        work.probe_turned_y[i].set_bits(oy);
        race.physics.contact.probe_turned_z_last.set_bits(oz);
        work.probe_turned_z[i].set_bits(oz);
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
        race.course
            .course
            .ground_face_last
            .set(car.probe_point_face[b as usize].get());
        q.settle_mark.set(0x80);
        ground_search_at(race, q, scene)?;
        work.probe_ground_height[i].set(0);
        if q.found.get() == 0 {
            race.physics.contact.crossed_probe.set(b);
            work.probe_found[i].set(0);
            continue;
        }
        work.probe_found[i].set(1);
        work.probe_ground_height[i].set(q.height.get());
        let cell = ground::noise_cell(q.x.get(), q.z.get());
        race.physics.contact.ground_cell_row[0].set(cell.row);
        race.physics.contact.ground_cell_row[1].set(cell.row_offset);
        let trap = crate::physics::cells::Trap::new();
        let noise = |i: u32| {
            trap.keep(
                crate::scene::engine_tables_store::ENGINE_TABLES
                    .noise_bytes
                    .read_at(i as usize, 1, "the ground noise table")
                    .map(|b| u32::from(b[0])),
            )
        };
        let n = cell.corners.map(noise);
        trap.finish()?;
        for (c, x) in [0, 1, 3, 2].into_iter().zip(n) {
            work.ground_noise_corners[c].set(x);
        }
        let (n58, n54, n5c, _) = ground::bump_noise(n, q.x.get(), q.z.get());
        race.physics.contact.ground_noise_mix[1].set(n58);
        race.physics.contact.ground_noise_mix[0].set(n54);
        race.physics.contact.ground_noise_mix[2].set(n5c);
        scratch.set(n5c);
        let bump = int_times_times_over_to_int(
            n5c as i32,
            car.ground_bump_scale.bits(),
            race.course.numbers.k256.bits(),
            race.course.numbers.hundred.bits(),
        );
        scratch.set(bump as u32);
        work.probe_ground_height[i]
            .set(work.probe_ground_height[i].get().wrapping_add(bump as u32));
        work.probe_face_flip[i].set(q.half.get());
        work.probe_share_u[i].set(q.plane_f.bits());
        work.probe_share_v[i].set(q.plane_g.bits());
        let d = work.probe_y[i]
            .get()
            .wrapping_sub(work.probe_ground_height[i].get());
        work.probe_height[i].set(d);
        if b < 4 {
            scratch.set(x87::loaded(car.crash_matrix[5].bits()));
            if x87::compare_status(scratch.get(), race.course.numbers.zero.bits()) & 0x100 != 0 {
                scratch.set(x87::loaded(race.course.numbers.zero.bits()));
            }
            work.wheel_height_raw[i].set(d);
            work.probe_height[i].set(x87::int_times_to_int(d as i32, scratch.get()) as u32);
        }
        car.probe_point_face[b as usize].set(q.slot.get());
    }
    for i in 0..4 {
        car.wheel_ground_x[i].set(race.physics.contact.probe_x[i].get());
        car.wheel_ground_height[i].set(race.physics.contact.probe_ground_height[i].get());
        car.wheel_ground_z[i].set(race.physics.contact.probe_z[i].get());
    }
    Ok(())
}

/// The Rust version.
/// `car` is the work car in the race's ticks, the view's car in the drawing.
pub fn probe_ground(
    race: &Race,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    probe_ground_at(
        race,
        car,
        scene,
        &GroundQuery::of_physics(race),
        &scene.drawing.scene.scratch,
    )
}

/// The same for the drawing (`race::face_hide_mask`): its ground search on fields of its
/// own and its work values in a local, not in the physics' work and `g_scratch` (nothing
/// the drawing does reads them after). It still writes the physics' contact work
/// (`race.physics.contact.probe_*`).
pub fn probe_ground_in_view(
    race: &Race,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let ground = crate::physics::ground_query::GroundCells::new();
    probe_ground_at(
        race,
        car,
        scene,
        &ground.query(),
        &crate::session::Dword::new(0),
    )
}
