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
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::flip_cell_index::flip_cell_index;
use crate::physics::step_families::WheelInputs;
use crate::physics::tables_store::Surface;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use std::cmp::Ordering;

/// Byte `offset` of texture info `n`.
fn info_byte(
    tables: &crate::scene::polygon_store::PolygonTables,
    n: u32,
    offset: u32,
) -> Result<u32, Refused> {
    Ok(u32::from(crate::scene::polygon::texture_info_byte(
        tables, n, offset,
    )?))
}

/// `v[c]` x 65536 rounded through `scratch` (`v` one of the probe shares).
fn share_bit(
    course: &Course,
    v: &[PlainFloat; 64],
    c: u32,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    x87_1(course, v, c as usize, scene);
    scene.drawing.scene.get().scratch.get()
}

/// Everything the function reads that the checks need, refused as said.
fn check(course: &Course, car: &Car) -> Result<(), Refused> {
    if !crate::race::course_places::held(&course.stage.course) {
        return Err(BadInput("no course faces").into());
    }
    if car.tyre_kind.get() > 5 {
        return Err(BadInput("a surface table index past 5").into());
    }
    Ok(())
}

/// The steps (a refusal may come after some are written: `wheel_surfaces` puts them
/// back).
fn steps(c: &WheelInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let contact = c.contact;
    check(c.course, c.car)?;
    for point in 0..5u32 {
        let cc = point as usize;
        let f = c.car.probe_point_face[cc].get();
        let face_byte = |offset| -> Result<u32, Refused> {
            Ok(u32::from(crate::race::course_places::face_field::<u8>(
                c.course.reads(),
                f,
                offset,
            )?))
        };
        let t1 = face_byte(1)?;
        let mut edx = share_bit(c.course, &contact.probe_share_v, point, scene);
        let mut ebx = share_bit(c.course, &contact.probe_share_u, point, scene);
        edx = (edx >> 15) & 1;
        ebx = (ebx >> 15) & 1;
        if contact.probe_face_flip[cc].get() == 1 {
            edx ^= 1;
            ebx ^= 1;
        }
        let (_, _, d1) = flip_cell_index(face_byte(0)?, ebx, edx);
        c.tables.surface_firsts[cc].set(d1);
        let (_, _, d2) = flip_cell_index(face_byte(0x20)?, ebx, edx);
        c.tables.surface_seconds[cc].set(d2);
        {
            let e = c.tables.surface_firsts[cc].get();
            c.car.wheel_surface_a[cc].set(info_byte(
                &c.shared.buffers.reads.get().polygon_tables,
                t1,
                e.wrapping_add(0x10),
            )?);
            c.car.wheel_cell_a_byte18[cc].set(info_byte(
                &c.shared.buffers.reads.get().polygon_tables,
                t1,
                e.wrapping_add(0x18),
            )?);
            c.car.wheel_cell_a_byte14[cc].set(info_byte(
                &c.shared.buffers.reads.get().polygon_tables,
                t1,
                e.wrapping_add(0x14),
            )?);
            let t2 = face_byte(0x21)?;
            let e2 = c.tables.surface_seconds[cc].get();
            c.car.wheel_surface_b[cc].set(info_byte(
                &c.shared.buffers.reads.get().polygon_tables,
                t2,
                e2.wrapping_add(0x10),
            )?);
            c.car.wheel_cell_b_byte18[cc].set(info_byte(
                &c.shared.buffers.reads.get().polygon_tables,
                t2,
                e2.wrapping_add(0x18),
            )?);
            c.car.wheel_cell_b_byte14[cc].set(info_byte(
                &c.shared.buffers.reads.get().polygon_tables,
                t2,
                e2.wrapping_add(0x14),
            )?);
            c.car.wheel_surface_blend[cc].set(0.0);
            let kind_index = info_byte(&c.shared.buffers.reads.get().polygon_tables, t2, 0x2C)?;
            // The index is a byte: past the four kinds it reads on (a slip kept as is).
            let kind = c.view.surface_kind(kind_index as usize)?;
            if kind != 5
                && kind != 4
                && c.car.wheel_surface_b[cc].get() != 0
                && face_byte(0x21)? != 0
            {
                let (x, y, z) = if contact.probe_face_flip[cc].get() == 1 {
                    (face_byte(0x25)?, face_byte(0x24)?, face_byte(0x23)?)
                } else {
                    (face_byte(0x22)?, face_byte(0x23)?, face_byte(0x24)?)
                };
                drawing.scene.get().surface_point[0].set(x);
                drawing.scene.get().surface_point[1].set(y.wrapping_sub(x));
                drawing.scene.get().surface_point[2].set(z.wrapping_sub(x));
                x87_2(contact, c.numbers, c.car, cc, scene);
            }
            if crate::race::course_places::face_number(c.course.reads(), f)?.switches() & 0x1000
                != 0
            {
                c.car.wheel_surface_a[cc].set(5);
                c.car.wheel_surface_b[cc].set(5);
                c.car.wheel_cell_a_byte18[cc].set(0x3C);
                c.car.wheel_cell_b_byte18[cc].set(0x3C);
                c.car.wheel_cell_a_byte14[cc].set(0);
                c.car.wheel_cell_b_byte14[cc].set(0);
            }
            let (s1, s2) = (
                c.car.wheel_surface_a[cc].get(),
                c.car.wheel_surface_b[cc].get(),
            );
            if s1 > 15 || s2 > 15 {
                return Err(BadInput("a surface past 15").into());
            }
            let (esi, edi) = (
                &c.tables.surfaces[s1 as usize],
                &c.tables.surfaces[s2 as usize],
            );
            x87_3(c.shared, esi, edi, cc);
            let (ps, pd) = (
                c.tables.surface_table_index(s1)?,
                c.tables.surface_table_index(s2)?,
            );
            let k = c.car.tyre_kind.get();
            x87_4(
                c.shared,
                c.numbers,
                &c.tables.surface_tables[ps..],
                &c.tables.surface_tables[pd..],
                k as usize,
            );
            x87_5_grips(c.shared, esi, edi, cc);
            // Only the centre point's (4) drag, minimum speed and mud blends are read (`x87_7`).
            if cc == 4 {
                x87_5_blends(c.shared, esi, edi, cc);
            }
        }
    }
    x87_6(c.shared);
    for point in 0..5usize {
        let hard = |s: u32| matches!(s, 1 | 2 | 3 | 0xC);
        let on =
            hard(c.car.wheel_surface_a[point].get()) || hard(c.car.wheel_surface_b[point].get());
        c.tables.hard_ground[point].set(on as u32);
    }
    let hard = c.tables.hard_ground[0]
        .get()
        .wrapping_add(c.tables.hard_ground[1].get())
        .wrapping_add(c.tables.hard_ground[2].get())
        .wrapping_add(c.tables.hard_ground[3].get());
    c.car.hard_ground_count.set(hard);
    let below =
        |a: f32, b: f32| !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal));
    let mut least = if below(c.tables.point_grips[0].get(), c.tables.point_grips[1].get()) {
        c.tables.point_grips[0].get()
    } else {
        c.tables.point_grips[1].get()
    };
    c.car.ground_bump_scale.set(crate::math::x87::loaded(least));
    for k in &c.tables.point_grips[2..5] {
        least = if below(c.car.ground_bump_scale.get(), k.get()) {
            c.car.ground_bump_scale.get()
        } else {
            k.get()
        };
        c.car.ground_bump_scale.set(crate::math::x87::loaded(least));
    }
    x87_7(c, scene);
    Ok(())
}

/// What was written is put back on a refusal.
pub fn wheel_surfaces(
    c: &WheelInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    steps(c, scene)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = [t + c * 4] * [k65536]`
fn x87_1(course: &Course, t: &[PlainFloat; 64], c: usize, scene: &crate::scene::owner::Scene) {
    let t0 = t[c].ld();
    let t1 = course.numbers.get().k65536.ld();
    let t2 = t0 * t1;
    scene.drawing.scene.get().scratch.set(to_int(t2) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables named below (`[name]`: a
/// float; `int [..]`: a whole number; `ext [..]`: 80 bits): A long formula, covered by the tests.
fn x87_2(
    contact: &ContactWork,
    numbers: &Numbers,
    car: &Car,
    c: usize,
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    let t0 = int(drawing.scene.get().surface_point[1].get() as i32);
    let t1 = contact.probe_share_u[c].ld();
    let t2 = t0 * t1;
    let t3 = int(drawing.scene.get().surface_point[2].get() as i32);
    let t4 = contact.probe_share_v[c].ld();
    let t5 = t3 * t4;
    let t6 = int(drawing.scene.get().surface_point[0].get() as i32);
    let t7 = t5 + t6;
    let t8 = t2 + t7;
    let t9 = numbers.k256.ld();
    let t10 = t8 / t9;
    car.wheel_surface_blend[c].st(t10);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_3(shared: &RaceShared, s: &Surface, d: &Surface, c: usize) {
    let t0 = s.grip.ld();
    let t1 = d.grip.ld();
    let t2 = t1 - t0;
    let t3 = shared.car().wheel_surface_blend[c].ld();
    let t4 = t2 * t3;
    let t5 = t0 + t4;
    shared.physics_tables.get().point_grips[c].st(t5);
    let t6 = s.wear_factor.ld();
    let t7 = d.wear_factor.ld();
    let t8 = t7 - t6;
    let t9 = shared.car().wheel_surface_blend[c].ld();
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    shared.car().point_wear_factor[c].st(t11);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_4(shared: &RaceShared, numbers: &Numbers, s: &[PlainDword], d: &[PlainDword], k: usize) {
    let t6 = int(s[k + 1].get() as i32);
    let t7 = int(s[k + 7].get() as i32);
    let t8 = t7 - t6;
    let t9 = shared.car().surface_blend.ld();
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    let t12 = numbers.hundredth_a.ld();
    let t13 = t11 * t12;
    shared.physics_tables.get().surface_scales[0].st(t13);
    let t14 = int(d[k + 1].get() as i32);
    let t15 = int(d[k + 7].get() as i32);
    let t16 = t15 - t14;
    let t17 = shared.car().surface_blend.ld();
    let t18 = t16 * t17;
    let t19 = t14 + t18;
    let t20 = numbers.hundredth_a.ld();
    let t21 = t19 * t20;
    shared.physics_tables.get().surface_scales[1].st(t21);
}

/// One straight x87 run as plain Rust over `X`.
#[cfg(test)]
fn x87_5(shared: &RaceShared, s: &Surface, d: &Surface, c: usize) {
    x87_5_grips(shared, s, d, c);
    x87_5_blends(shared, s, d, c);
}

/// `x87_5`'s first three blends (the point's three grip figures), needed for every point.
fn x87_5_grips(shared: &RaceShared, s: &Surface, d: &Surface, c: usize) {
    let t0 = s.slide_grip.ld();
    let t1 = shared.physics_tables.get().surface_scales[0].ld();
    let t2 = t0 * t1;
    let t3 = d.slide_grip.ld();
    let t4 = shared.physics_tables.get().surface_scales[1].ld();
    let t5 = t3 * t4;
    let t6 = t5 - t2;
    let t7 = shared.car().wheel_surface_blend[c].ld();
    let t8 = t6 * t7;
    let t9 = t2 + t8;
    let t10 = shared.physics_tables.get().spec_surface[2].ld();
    let t11 = t9 * t10;
    shared.physics_tables.get().wheel_loads[c].st(t11);
    let t12 = s.accel_grip.ld();
    let t13 = shared.physics_tables.get().surface_scales[0].ld();
    let t14 = t12 * t13;
    let t15 = d.accel_grip.ld();
    let t16 = shared.physics_tables.get().surface_scales[1].ld();
    let t17 = t15 * t16;
    let t18 = t17 - t14;
    let t19 = shared.car().wheel_surface_blend[c].ld();
    let t20 = t18 * t19;
    let t21 = t14 + t20;
    let t22 = shared.physics_tables.get().spec_surface[0].ld();
    let t23 = t21 * t22;
    shared.physics_tables.get().wheel_grips[c].st(t23);
    let t24 = s.brake_grip.ld();
    let t25 = shared.physics_tables.get().surface_scales[0].ld();
    let t26 = t24 * t25;
    let t27 = d.brake_grip.ld();
    let t28 = shared.physics_tables.get().surface_scales[1].ld();
    let t29 = t27 * t28;
    let t30 = t29 - t26;
    let t31 = shared.car().wheel_surface_blend[c].ld();
    let t32 = t30 * t31;
    let t33 = t26 + t32;
    let t34 = shared.physics_tables.get().spec_surface[1].ld();
    let t35 = t33 * t34;
    shared.physics_tables.get().wheel_grips_b[c].st(t35);
}

/// `x87_5`'s last four blends (`surface_blends`: drag, minimum speed, two mud rates), read only
/// for the centre point (4) by `x87_7`; the other points' are not computed.
fn x87_5_blends(shared: &RaceShared, s: &Surface, d: &Surface, c: usize) {
    let t36 = s.drag.ld();
    let t37 = d.drag.ld();
    let t38 = t37 - t36;
    let t39 = shared.car().wheel_surface_blend[c].ld();
    let t40 = t38 * t39;
    let t41 = t36 + t40;
    shared.physics_tables.get().surface_blends[c].st(t41);
    let t42 = s.rolling_drag_min_speed.ld();
    let t43 = d.rolling_drag_min_speed.ld();
    let t44 = t43 - t42;
    let t45 = shared.car().wheel_surface_blend[c].ld();
    let t46 = t44 * t45;
    let t47 = t42 + t46;
    shared.physics_tables.get().surface_blends[5 + c].st(t47);
    let t48 = s.mud_rates[1].ld();
    let t49 = d.mud_rates[1].ld();
    let t50 = t49 - t48;
    let t51 = shared.car().wheel_surface_blend[c].ld();
    let t52 = t50 * t51;
    let t53 = t48 + t52;
    shared.physics_tables.get().surface_blends[10 + c].st(t53);
    let t54 = s.mud_rates[0].ld();
    let t55 = d.mud_rates[0].ld();
    let t56 = t55 - t54;
    let t57 = shared.car().wheel_surface_blend[c].ld();
    let t58 = t56 * t57;
    let t59 = t54 + t58;
    shared.physics_tables.get().surface_blends[15 + c].st(t59);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_6(shared: &RaceShared) {
    let t0 = shared.physics_tables.get().wheel_loads[0].ld();
    let t1 = shared.physics_tables.get().wheel_loads[1].ld();
    let t2 = t0 + t1;
    let t3 = shared.physics_tables.get().wheel_loads[2].ld();
    let t4 = t2 + t3;
    let t5 = shared.physics_tables.get().wheel_loads[3].ld();
    let t6 = t4 + t5;
    let t7 = shared.physics_tables.get().wheel_load_scale.ld();
    let t8 = t6 * t7;
    shared.physics_tables.get().wheel_load.st(t8);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_7(c: &WheelInputs<'_>, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = c.car.ground_bump_scale.ld();
    let t1 = c.tables.handling[6].ld();
    let t2 = t0 * t1;
    c.car.ground_bump_scale.st(t2);
    let t3 = c.tables.surface_blend[0].ld();
    let t4 = c.tables.surface_blend[2].ld();
    let t5 = t3 - t4;
    let t6 = c.car.surface_blend.ld();
    let t7 = t5 * t6;
    let t8 = c.tables.surface_blend[2].ld();
    let t9 = t7 + t8;
    drawing.scene.get().surface_point[0].st(t9);
    let t10 = c.tables.surface_blend[1].ld();
    let t11 = c.tables.surface_blend[3].ld();
    let t12 = t10 - t11;
    let t13 = c.car.surface_blend.ld();
    let t14 = t12 * t13;
    let t15 = c.tables.surface_blend[3].ld();
    let t16 = t14 + t15;
    drawing.scene.get().surface_point[1].st(t16);
    let t17 = drawing.scene.get().surface_point[1].ld();
    let t18 = drawing.scene.get().surface_point[0].ld();
    let t19 = t17 - t18;
    let t20 = int(c.car.suspension_stiffness.get() as i32);
    let t21 = c.numbers.k256.ld();
    let t22 = t20 / t21;
    let t23 = t19 * t22;
    let t24 = drawing.scene.get().surface_point[0].ld();
    let t25 = t23 + t24;
    c.tables.surface_softness.st(t25);
    let t26 = c.tables.surface_blends[4].ld();
    c.tables.wheel_drag[0].st(t26);
    let t27 = c.tables.surface_blends[9].ld();
    c.tables.wheel_drag[1].st(t27);
    let t28 = c.tables.surface_blends[14].ld();
    c.vars.mud_scale_a[0].st(t28);
    let t29 = c.tables.surface_blends[19].ld();
    c.vars.mud_scale_a[1].st(t29);
}

#[cfg(test)]
mod tests;
