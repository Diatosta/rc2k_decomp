//! The surface under each of the five ground points (`probe_point_face[c]`, their faces) and the
//! grip figures from it.
//!
//! For each point c: the face and its texture's record (the texture info records, by the face's
//! texture byte); the cell (the point's shares `probe_share_v[c]` / `probe_share_u[c]` x 65536,
//! rounded, bit 15, flipped with `probe_face_flip[c]` 1) turned by the face's two flip bytes
//! (`physics::flip_cell_index`) into `surface_firsts[c]` / `surface_seconds[c]`; the two textures'
//! surface kinds and two more bytes by those cells into the `wheel_surface_a` / `wheel_cell_a_*`
//! and `wheel_surface_b` / `wheel_cell_b_*` fields; the blend `wheel_surface_blend[c]` 0, or
//! (unless the second texture's kind (`surface_kinds`) is 4 or 5, its surface 0, or the face's
//! second texture number 0) the face's three blend bytes by the shares / 256; with bit 0x1000 of
//! the face's switches both surfaces 5 (wet, by the look) and the bytes 0x3C, 0. Then each figure
//! is blended between the two surface records (`surfaces`, 0x30 each): grip, two more, the record's
//! scaled tables by the car's `tyre_kind` and `surface_blend`, into `point_grips`,
//! `point_wear_factor`, `wheel_loads`, `wheel_grips`, `wheel_grips_b`, `surface_blends` (the last
//! four only for the centre point, the only one read).
//!
//! After the five: `wheel_load` = (the first four `wheel_loads`) x `wheel_load_scale`; the count of
//! points on hard surfaces (1, 2, 3, 0xC) into `hard_ground_count` (`hard_ground`);
//! `ground_bump_scale` the least of the four grips x `handling[6]`; `surface_softness` by
//! `surface_blend` and `suspension_stiffness`; four figures copied on.
//!
//! The x87 runs are plain Rust over `X` (`math/x87_float.rs`); the integer steps are Rust.
//!
//! Refuses no faces, a table index (`tyre_kind`) past 5 (before anything) and a surface past 15
//! (what was written by then is put back).

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::cell_x87::PlainX87;
use crate::math::x87_float::to_int;
use crate::math::x87_float::{int, ld, st};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ProbeLists;
use crate::physics::figures::SURFACE_TABLES;
use crate::physics::flip_cell_index::flip_cell_index;
use crate::physics::step_families::WheelInputs;
use crate::physics::tables_store::Surface;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use std::cmp::Ordering;

/// Byte `offset` of texture info `n`.
fn info_byte(
    tables: crate::scene::polygon_store::PolygonTablesRef,
    n: u32,
    offset: u32,
) -> Result<u32, Refused> {
    Ok(u32::from(crate::scene::polygon::texture_info_byte(
        tables, n, offset,
    )?))
}

/// `v[c]` x 65536 rounded to a whole number (`v` one of the probe shares).
fn share_bit(numbers: &Numbers, v: &[f32; 64], c: u32) -> u32 {
    x87_1(numbers, v, c as usize)
}

/// Everything the function reads that the checks need, refused as said.
fn check(course: &Course, car: &Car) -> Result<(), Refused> {
    if !crate::race::course_places::held(&course.stage.course) {
        return Err(BadInput("no course faces").into());
    }
    if car.tyre_kind > 5 {
        return Err(BadInput("a surface table index past 5").into());
    }
    Ok(())
}

/// The steps (a refusal may come after some are written: `wheel_surfaces` puts them
/// back).
fn steps(
    c: &WheelInputs<'_>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<(), Refused> {
    check(c.course, car)?;
    for point in 0..5u32 {
        let cc = point as usize;
        let f = car.probe_point_face[cc];
        let face_byte = |offset| -> Result<u32, Refused> {
            Ok(u32::from(crate::race::course_places::face_field::<u8>(
                c.course.reads(),
                f,
                offset,
            )?))
        };
        let t1 = face_byte(1)?;
        let probes = c.probes.borrow();
        let mut share_v = share_bit(&c.course.numbers, &probes.probe_share_v, point);
        let mut share_u = share_bit(&c.course.numbers, &probes.probe_share_u, point);
        share_v = (share_v >> 15) & 1;
        share_u = (share_u >> 15) & 1;
        if probes.probe_face_flip[cc] == 1 {
            share_v ^= 1;
            share_u ^= 1;
        }
        let (_, _, d1) = flip_cell_index(face_byte(0)?, share_u, share_v);
        c.tables.borrow_mut().surface_firsts[cc] = d1;
        let (_, _, d2) = flip_cell_index(face_byte(0x20)?, share_u, share_v);
        c.tables.borrow_mut().surface_seconds[cc] = d2;
        {
            let e = c.tables.borrow().surface_firsts[cc];
            car.wheel_surface_a[cc] =
                info_byte(c.shared.buffers.polygon_tables(), t1, e.wrapping_add(0x10))?;
            car.wheel_cell_a_byte18[cc] =
                info_byte(c.shared.buffers.polygon_tables(), t1, e.wrapping_add(0x18))?;
            car.wheel_cell_a_byte14[cc] =
                info_byte(c.shared.buffers.polygon_tables(), t1, e.wrapping_add(0x14))?;
            let t2 = face_byte(0x21)?;
            let e2 = c.tables.borrow().surface_seconds[cc];
            car.wheel_surface_b[cc] =
                info_byte(c.shared.buffers.polygon_tables(), t2, e2.wrapping_add(0x10))?;
            car.wheel_cell_b_byte18[cc] =
                info_byte(c.shared.buffers.polygon_tables(), t2, e2.wrapping_add(0x18))?;
            car.wheel_cell_b_byte14[cc] =
                info_byte(c.shared.buffers.polygon_tables(), t2, e2.wrapping_add(0x14))?;
            car.wheel_surface_blend[cc] = 0.0;
            let kind_index = info_byte(c.shared.buffers.polygon_tables(), t2, 0x2C)?;
            // The index is a byte: past the four kinds it reads on (a slip kept as is).
            let kind = c.view.surface_kind(kind_index as usize)?;
            if kind != 5 && kind != 4 && car.wheel_surface_b[cc] != 0 && face_byte(0x21)? != 0 {
                let (x, y, z) = if probes.probe_face_flip[cc] == 1 {
                    (face_byte(0x25)?, face_byte(0x24)?, face_byte(0x23)?)
                } else {
                    (face_byte(0x22)?, face_byte(0x23)?, face_byte(0x24)?)
                };
                scene.draw_work_mut().surface_point[0] = x;
                scene.draw_work_mut().surface_point[1] = y.wrapping_sub(x);
                scene.draw_work_mut().surface_point[2] = z.wrapping_sub(x);
                x87_2(&probes, c.numbers, car, cc, scene);
            }
            if crate::race::course_places::face_number(c.course.reads(), f)?.switches() & 0x1000
                != 0
            {
                car.wheel_surface_a[cc] = 5;
                car.wheel_surface_b[cc] = 5;
                car.wheel_cell_a_byte18[cc] = 0x3C;
                car.wheel_cell_b_byte18[cc] = 0x3C;
                car.wheel_cell_a_byte14[cc] = 0;
                car.wheel_cell_b_byte14[cc] = 0;
            }
            let (s1, s2) = (car.wheel_surface_a[cc], car.wheel_surface_b[cc]);
            if s1 > 15 || s2 > 15 {
                return Err(BadInput("a surface past 15").into());
            }
            // Copies: the figures are blended into the tables' other fields.
            let (esi, edi) = {
                let tables = c.tables.borrow();
                (
                    tables.surfaces[s1 as usize].clone(),
                    tables.surfaces[s2 as usize].clone(),
                )
            };
            x87_3(c.shared, car, &esi, &edi, cc);
            let (ps, pd) = (
                c.tables.borrow().surface_table_index(s1)?,
                c.tables.borrow().surface_table_index(s2)?,
            );
            let k = car.tyre_kind;
            x87_4(
                c.shared,
                car,
                c.numbers,
                &SURFACE_TABLES[ps..],
                &SURFACE_TABLES[pd..],
                k as usize,
            );
            x87_5_grips(c.shared, car, &esi, &edi, cc);
            // Only the centre point's (4) drag, minimum speed and mud blends are read (`x87_7`).
            if cc == 4 {
                x87_5_blends(c.shared, car, &esi, &edi, cc);
            }
        }
    }
    x87_6(c.numbers, c.shared);
    for point in 0..5usize {
        let hard = |s: u32| matches!(s, 1 | 2 | 3 | 0xC);
        let on = hard(car.wheel_surface_a[point]) || hard(car.wheel_surface_b[point]);
        c.tables.borrow_mut().hard_ground[point] = on as u32;
    }
    let hard = {
        let tables = c.tables.borrow();
        tables.hard_ground[0]
            .wrapping_add(tables.hard_ground[1])
            .wrapping_add(tables.hard_ground[2])
            .wrapping_add(tables.hard_ground[3])
    };
    car.hard_ground_count = hard;
    let below =
        |a: f32, b: f32| !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal));
    let grips = c.tables.borrow().point_grips;
    let mut least = if below(grips[0], grips[1]) {
        grips[0]
    } else {
        grips[1]
    };
    car.ground_bump_scale = crate::math::x87::loaded(least);
    let grips = c.tables.borrow().point_grips;
    for &k in &grips[2..5] {
        least = if below(car.ground_bump_scale, k) {
            car.ground_bump_scale
        } else {
            k
        };
        car.ground_bump_scale = crate::math::x87::loaded(least);
    }
    x87_7(c, scene, car);
    Ok(())
}

/// What was written is put back on a refusal.
pub fn wheel_surfaces(
    c: &WheelInputs<'_>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<(), Refused> {
    steps(c, scene, car)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [result] = [t + c * 4] * [k65536]`
fn x87_1(numbers: &Numbers, t: &[f32; 64], c: usize) -> u32 {
    let t0 = ld(t[c]);
    let t1 = numbers.k65536.ld();
    let t2 = t0 * t1;
    to_int(t2) as u32
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables named below (`[name]`: a
/// float; `int [..]`: a whole number; `ext [..]`: 80 bits): A long formula, covered by the tests.
fn x87_2(
    probes: &ProbeLists,
    numbers: &Numbers,
    car: &mut Car,
    c: usize,
    scene: &crate::scene::owner::Scene,
) {
    let t0 = int(scene.draw_work().surface_point[1] as i32);
    let t1 = ld(probes.probe_share_u[c]);
    let t2 = t0 * t1;
    let t3 = int(scene.draw_work().surface_point[2] as i32);
    let t4 = ld(probes.probe_share_v[c]);
    let t5 = t3 * t4;
    let t6 = int(scene.draw_work().surface_point[0] as i32);
    let t7 = t5 + t6;
    let t8 = t2 + t7;
    let t9 = numbers.k256.ld();
    let t10 = t8 / t9;
    car.wheel_surface_blend[c].st(t10);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_3(shared: &RaceShared, car: &mut Car, s: &Surface, d: &Surface, c: usize) {
    let t0 = s.grip.ld();
    let t1 = d.grip.ld();
    let t2 = t1 - t0;
    let t3 = car.wheel_surface_blend[c].ld();
    let t4 = t2 * t3;
    let t5 = t0 + t4;
    shared.physics_tables_mut().point_grips[c].st(t5);
    let t6 = s.wear_factor.ld();
    let t7 = d.wear_factor.ld();
    let t8 = t7 - t6;
    let t9 = car.wheel_surface_blend[c].ld();
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    car.point_wear_factor[c].st(t11);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_4(shared: &RaceShared, car: &Car, numbers: &Numbers, s: &[u32], d: &[u32], k: usize) {
    let t6 = int(s[k + 1] as i32);
    let t7 = int(s[k + 7] as i32);
    let t8 = t7 - t6;
    let t9 = car.surface_blend.ld();
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    let t12 = numbers.hundredth_a.ld();
    let t13 = t11 * t12;
    shared.physics_tables_mut().surface_scales[0].st(t13);
    let t14 = int(d[k + 1] as i32);
    let t15 = int(d[k + 7] as i32);
    let t16 = t15 - t14;
    let t17 = car.surface_blend.ld();
    let t18 = t16 * t17;
    let t19 = t14 + t18;
    let t20 = numbers.hundredth_a.ld();
    let t21 = t19 * t20;
    shared.physics_tables_mut().surface_scales[1].st(t21);
}

/// One straight x87 run as plain Rust over `X`.
#[cfg(test)]
fn x87_5(shared: &RaceShared, car: &Car, s: &Surface, d: &Surface, c: usize) {
    x87_5_grips(shared, car, s, d, c);
    x87_5_blends(shared, car, s, d, c);
}

/// `x87_5`'s first three blends (the point's three grip figures), needed for every point.
fn x87_5_grips(shared: &RaceShared, car: &Car, s: &Surface, d: &Surface, c: usize) {
    let t0 = s.slide_grip.ld();
    let t1 = shared.physics_tables().surface_scales[0].ld();
    let t2 = t0 * t1;
    let t3 = d.slide_grip.ld();
    let t4 = shared.physics_tables().surface_scales[1].ld();
    let t5 = t3 * t4;
    let t6 = t5 - t2;
    let t7 = car.wheel_surface_blend[c].ld();
    let t8 = t6 * t7;
    let t9 = t2 + t8;
    let t10 = shared.physics_tables().spec_surface[2].ld();
    let t11 = t9 * t10;
    shared.physics_tables_mut().wheel_loads[c].st(t11);
    let t12 = s.accel_grip.ld();
    let t13 = shared.physics_tables().surface_scales[0].ld();
    let t14 = t12 * t13;
    let t15 = d.accel_grip.ld();
    let t16 = shared.physics_tables().surface_scales[1].ld();
    let t17 = t15 * t16;
    let t18 = t17 - t14;
    let t19 = car.wheel_surface_blend[c].ld();
    let t20 = t18 * t19;
    let t21 = t14 + t20;
    let t22 = shared.physics_tables().spec_surface[0].ld();
    let t23 = t21 * t22;
    shared.physics_tables_mut().wheel_grips[c].st(t23);
    let t24 = s.brake_grip.ld();
    let t25 = shared.physics_tables().surface_scales[0].ld();
    let t26 = t24 * t25;
    let t27 = d.brake_grip.ld();
    let t28 = shared.physics_tables().surface_scales[1].ld();
    let t29 = t27 * t28;
    let t30 = t29 - t26;
    let t31 = car.wheel_surface_blend[c].ld();
    let t32 = t30 * t31;
    let t33 = t26 + t32;
    let t34 = shared.physics_tables().spec_surface[1].ld();
    let t35 = t33 * t34;
    shared.physics_tables_mut().wheel_grips_b[c].st(t35);
}

/// `x87_5`'s last four blends (`surface_blends`: drag, minimum speed, two mud rates), read only
/// for the centre point (4) by `x87_7`; the other points' are not computed.
fn x87_5_blends(shared: &RaceShared, car: &Car, s: &Surface, d: &Surface, c: usize) {
    let t36 = s.drag.ld();
    let t37 = d.drag.ld();
    let t38 = t37 - t36;
    let t39 = car.wheel_surface_blend[c].ld();
    let t40 = t38 * t39;
    let t41 = t36 + t40;
    shared.physics_tables_mut().surface_blends[c].st(t41);
    let t42 = s.rolling_drag_min_speed.ld();
    let t43 = d.rolling_drag_min_speed.ld();
    let t44 = t43 - t42;
    let t45 = car.wheel_surface_blend[c].ld();
    let t46 = t44 * t45;
    let t47 = t42 + t46;
    shared.physics_tables_mut().surface_blends[5 + c].st(t47);
    let t48 = s.mud_rates[1].ld();
    let t49 = d.mud_rates[1].ld();
    let t50 = t49 - t48;
    let t51 = car.wheel_surface_blend[c].ld();
    let t52 = t50 * t51;
    let t53 = t48 + t52;
    shared.physics_tables_mut().surface_blends[10 + c].st(t53);
    let t54 = s.mud_rates[0].ld();
    let t55 = d.mud_rates[0].ld();
    let t56 = t55 - t54;
    let t57 = car.wheel_surface_blend[c].ld();
    let t58 = t56 * t57;
    let t59 = t54 + t58;
    shared.physics_tables_mut().surface_blends[15 + c].st(t59);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_6(numbers: &Numbers, shared: &RaceShared) {
    let t0 = shared.physics_tables().wheel_loads[0].ld();
    let t1 = shared.physics_tables().wheel_loads[1].ld();
    let t2 = t0 + t1;
    let t3 = shared.physics_tables().wheel_loads[2].ld();
    let t4 = t2 + t3;
    let t5 = shared.physics_tables().wheel_loads[3].ld();
    let t6 = t4 + t5;
    let t7 = numbers.wheel_load_scale.ld();
    let t8 = t6 * t7;
    shared.physics_tables_mut().wheel_load.st(t8);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_7(c: &WheelInputs<'_>, scene: &crate::scene::owner::Scene, car: &mut Car) {
    let t0 = car.ground_bump_scale.ld();
    let t1 = c.tables.borrow().handling[6].ld();
    let t2 = t0 * t1;
    car.ground_bump_scale.st(t2);
    let t3 = c.numbers.surface_blend[0].ld();
    let t4 = c.numbers.surface_blend[2].ld();
    let t5 = t3 - t4;
    let t6 = car.surface_blend.ld();
    let t7 = t5 * t6;
    let t8 = c.numbers.surface_blend[2].ld();
    let t9 = t7 + t8;
    scene.draw_work_mut().surface_point[0] = st(t9);
    let t10 = c.numbers.surface_blend[1].ld();
    let t11 = c.numbers.surface_blend[3].ld();
    let t12 = t10 - t11;
    let t13 = car.surface_blend.ld();
    let t14 = t12 * t13;
    let t15 = c.numbers.surface_blend[3].ld();
    let t16 = t14 + t15;
    scene.draw_work_mut().surface_point[1] = st(t16);
    let t17 = ld(scene.draw_work().surface_point[1]);
    let t18 = ld(scene.draw_work().surface_point[0]);
    let t19 = t17 - t18;
    let t20 = int(car.suspension_stiffness as i32);
    let t21 = c.numbers.k256.ld();
    let t22 = t20 / t21;
    let t23 = t19 * t22;
    let t24 = ld(scene.draw_work().surface_point[0]);
    let t25 = t23 + t24;
    c.tables.borrow_mut().surface_softness.st(t25);
    let t26 = c.tables.borrow().surface_blends[4].ld();
    c.tables.borrow_mut().wheel_drag[0].st(t26);
    let t27 = c.tables.borrow().surface_blends[9].ld();
    c.tables.borrow_mut().wheel_drag[1].st(t27);
    let t28 = c.tables.borrow().surface_blends[14].ld();
    c.vars.mud_scale_a[0].st(t28);
    let t29 = c.tables.borrow().surface_blends[19].ld();
    c.vars.mud_scale_a[1].st(t29);
}

#[cfg(test)]
mod tests;
