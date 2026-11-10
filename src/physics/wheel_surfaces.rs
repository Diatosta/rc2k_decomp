//! `physics::wheel_surfaces` (0x43D59A): the surface under each of the five ground points
//! (`unk_71ADC8[c]`, their faces) and the grip figures from it.
//!
//! For each point c: the face and its texture's record (`g_textureInfos`, by the face's
//! +1); the cell (the point's shares `dword_6D2B14[c]` / `dword_6D2A14[c]` x 65536,
//! `fistp`, bit 15, flipped with `dword_6D2914[c]` 1) turned by the face's +0 and +0x20
//! (`physics::flip_cell_index`) into `dword_614710[c]` / `dword_614724[c]`; the two
//! textures' surface kinds and two more bytes by those cells into +0x920 / +0x95C /
//! +0x984 [c] and +0x934 / +0x970 / +0x998 [c]; the blend +0x9AC [c] 0, or (unless the
//! second texture's kind (`dword_63D9D0`) is 4 or 5, its surface 0, or the face's +0x21
//! 0) the face's three blend bytes by the shares / 256; with bit 0x1000 of the face's +2
//! both surfaces 5 (wet, by the look) and the bytes 0x3C, 0. Then each figure blended
//! between the two surface records (`off_613978`, 0x30 each): grip, two more, the record's
//! scaled tables by `g_car` +0x2C and +0x280, into `flt_6142C0`, +0x948, `flt_614284`,
//! `flt_614298`, `flt_6142AC`, `flt_6146C0` .. [c] (the last four, `flt_6146C0` ..,
//! only for the centre point, the only one read).
//!
//! After the five: `flt_614268` = (the first four `flt_614284`) x `flt_613FEC`; the count
//! of points on hard surfaces (1, 2, 3, 0xC) into +0x258 (`dword_614150` ..); +0x60 the
//! least of the four grips x `flt_614C04`; `flt_614230` by +0x280 and +0x34; four
//! figures copied on.
//!
//! The x87 runs are plain Rust over `X` (`math/x87_float.rs`), the original's instructions
//! (from IDA) being `asm_reference` in the tests; the integer steps in Rust.
//!
//! Callers: `physics::car_physics_step` (0x43B70F), `drive::simple_car_step`. Refuses no faces, a
//! table index (+0x2C) past 5 (before anything) and a surface past 15 (what was written by then is
//! put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::flip_cell_index::flip_cell_index;
use crate::physics::tables_store::{PhysicsTables, Surface};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::Shared;
use crate::session::Dword;
use std::cmp::Ordering;

/// Byte `offset` of texture info `n`.
fn info_byte(n: u32, offset: u32) -> Result<u32, Refused> {
    Ok(u32::from(crate::scene::polygon::texture_info_byte(
        n, offset,
    )?))
}

/// `fld v[c] x 65536; fistp` through `dword_62400C` (`v` one of the probe shares).
fn share_bit(course: &Course, v: &[Dword; 64], c: u32, scene: &crate::scene::owner::Scene) -> u32 {
    x87_1(course, v, c as usize, scene);
    scene.drawing.scene.scratch.get()
}

/// Everything the function reads that the checks need, refused as said.
fn check(race: &Race) -> Result<(), Refused> {
    let faces = race.course.course.course_faces.get();
    if faces == 0 {
        return Err(BadInput("no course faces").into());
    }
    if race.shared.car.tyre_kind.get() > 5 {
        return Err(BadInput("a surface table index past 5").into());
    }
    Ok(())
}

/// The steps (a refusal may come after some are written: `wheel_surfaces` puts them
/// back).
fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let physics = &race.physics;
    check(race)?;
    for c in 0..5u32 {
        let cc = c as usize;
        let f = race.shared.car.probe_point_face[cc].get();
        let face_byte = |offset| -> Result<u32, Refused> {
            Ok(u32::from(crate::race::course_places::face_field::<u8>(
                f,
                offset,
                "a wheel's face",
            )?))
        };
        let t1 = face_byte(1)?;
        let mut edx = share_bit(&race.course, &physics.contact.probe_share_v, c, scene);
        let mut ebx = share_bit(&race.course, &physics.contact.probe_share_u, c, scene);
        edx = (edx >> 15) & 1;
        ebx = (ebx >> 15) & 1;
        if physics.contact.probe_face_flip[cc].get() == 1 {
            edx ^= 1;
            ebx ^= 1;
        }
        let (_, _, d1) = flip_cell_index(face_byte(0)?, ebx, edx);
        race.shared.physics_tables.surface_firsts[cc].set(d1);
        let (_, _, d2) = flip_cell_index(face_byte(0x20)?, ebx, edx);
        race.shared.physics_tables.surface_seconds[cc].set(d2);
        {
            let e = race.shared.physics_tables.surface_firsts[cc].get();
            race.shared.car.wheel_surface_a[cc].set(info_byte(t1, e.wrapping_add(0x10))?);
            race.shared.car.wheel_cell_a_byte18[cc].set(info_byte(t1, e.wrapping_add(0x18))?);
            race.shared.car.wheel_cell_a_byte14[cc].set(info_byte(t1, e.wrapping_add(0x14))?);
            let t2 = face_byte(0x21)?;
            let e2 = race.shared.physics_tables.surface_seconds[cc].get();
            race.shared.car.wheel_surface_b[cc].set(info_byte(t2, e2.wrapping_add(0x10))?);
            race.shared.car.wheel_cell_b_byte18[cc].set(info_byte(t2, e2.wrapping_add(0x18))?);
            race.shared.car.wheel_cell_b_byte14[cc].set(info_byte(t2, e2.wrapping_add(0x14))?);
            race.shared.car.wheel_surface_blend[cc].set_bits(0);
            let kind_index = info_byte(t2, 0x2C)?;
            // The index is a byte: past the four kinds the original reads on (a slip).
            let kind = crate::session::read_on_in(
                &race.shared.view,
                &race.shared.view.surface_kinds,
                kind_index as usize,
                "the surface kinds (a byte index past the table)",
            )?;
            if kind != 5
                && kind != 4
                && race.shared.car.wheel_surface_b[cc].get() != 0
                && face_byte(0x21)? != 0
            {
                let (x, y, z) = if physics.contact.probe_face_flip[cc].get() == 1 {
                    (face_byte(0x25)?, face_byte(0x24)?, face_byte(0x23)?)
                } else {
                    (face_byte(0x22)?, face_byte(0x23)?, face_byte(0x24)?)
                };
                drawing.scene.surface_point[0].set(x);
                drawing.scene.surface_point[1].set(y.wrapping_sub(x));
                drawing.scene.surface_point[2].set(z.wrapping_sub(x));
                x87_2(race, c as usize, scene);
            }
            if crate::race::course_places::face_number(f, "a wheel's face")?.switches() & 0x1000
                != 0
            {
                race.shared.car.wheel_surface_a[cc].set(5);
                race.shared.car.wheel_surface_b[cc].set(5);
                race.shared.car.wheel_cell_a_byte18[cc].set(0x3C);
                race.shared.car.wheel_cell_b_byte18[cc].set(0x3C);
                race.shared.car.wheel_cell_a_byte14[cc].set(0);
                race.shared.car.wheel_cell_b_byte14[cc].set(0);
            }
            let (s1, s2) = (
                race.shared.car.wheel_surface_a[cc].get(),
                race.shared.car.wheel_surface_b[cc].get(),
            );
            if s1 > 15 || s2 > 15 {
                return Err(BadInput("a surface past 15").into());
            }
            let (esi, edi) = (
                &race.shared.physics_tables.surfaces[s1 as usize],
                &race.shared.physics_tables.surfaces[s2 as usize],
            );
            x87_3(&race.shared, esi, edi, cc);
            let (ps, pd) = (
                PhysicsTables::surface_table_index(s1)?,
                PhysicsTables::surface_table_index(s2)?,
            );
            let k = race.shared.car.tyre_kind.get();
            x87_4(
                race,
                &race.shared.physics_tables.surface_tables[ps..],
                &race.shared.physics_tables.surface_tables[pd..],
                k as usize,
            );
            x87_5_grips(race, esi, edi, cc);
            // Only the centre point's (4) drag, minimum speed and mud blends are read (`x87_7`).
            if cc == 4 {
                x87_5_blends(race, esi, edi, cc);
            }
        }
    }
    x87_6(race);
    for c in 0..5usize {
        let hard = |s: u32| matches!(s, 1 | 2 | 3 | 0xC);
        let on = hard(race.shared.car.wheel_surface_a[c].get())
            || hard(race.shared.car.wheel_surface_b[c].get());
        race.shared.physics_tables.hard_ground[c].set(on as u32);
    }
    let hard = race.shared.physics_tables.hard_ground[0]
        .get()
        .wrapping_add(race.shared.physics_tables.hard_ground[1].get())
        .wrapping_add(race.shared.physics_tables.hard_ground[2].get())
        .wrapping_add(race.shared.physics_tables.hard_ground[3].get());
    race.shared.car.hard_ground_count.set(hard);
    let below = |a: u32, b: u32| {
        !matches!(
            f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
            Some(Ordering::Greater | Ordering::Equal)
        )
    };
    let mut least = if below(
        race.shared.physics_tables.point_grips[0].bits(),
        race.shared.physics_tables.point_grips[1].bits(),
    ) {
        race.shared.physics_tables.point_grips[0].bits()
    } else {
        race.shared.physics_tables.point_grips[1].bits()
    };
    race.shared
        .car
        .ground_bump_scale
        .set_bits(crate::math::x87::loaded(least));
    for k in &race.shared.physics_tables.point_grips[2..5] {
        least = if below(race.shared.car.ground_bump_scale.bits(), k.bits()) {
            race.shared.car.ground_bump_scale.bits()
        } else {
            k.bits()
        };
        race.shared
            .car
            .ground_bump_scale
            .set_bits(crate::math::x87::loaded(least));
    }
    x87_7(race, scene);
    Ok(())
}

/// What was written is put back on a refusal.
pub fn wheel_surfaces(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    steps(race, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = [t + c * 4] * [rc + 0x1C5D98]`
fn x87_1(course: &Course, t: &[Dword; 64], c: usize, scene: &crate::scene::owner::Scene) {
    let t0 = ld(t[c].bits());
    let t1 = ld(course.numbers.k65536.bits());
    let t2 = t0 * t1;
    scene.drawing.scene.scratch.set_bits(to_int(t2) as u32);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_2(race: &Race, c: usize, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = int(drawing.scene.surface_point[1].bits() as i32);
    let t1 = ld(race.physics.contact.probe_share_u[c].bits());
    let t2 = t0 * t1;
    let t3 = int(drawing.scene.surface_point[2].bits() as i32);
    let t4 = ld(race.physics.contact.probe_share_v[c].bits());
    let t5 = t3 * t4;
    let t6 = int(drawing.scene.surface_point[0].bits() as i32);
    let t7 = t5 + t6;
    let t8 = t2 + t7;
    let t9 = ld(race.course.numbers.k256.bits());
    let t10 = t8 / t9;
    race.shared.car.wheel_surface_blend[c].set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + c * 4 + 0x2142C0] = [s + 0x28] + (([d + 0x28] - [s + 0x28]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x31A920] = [s + 0x2C] + (([d + 0x2C] - [s + 0x2C]) * [rc + c * 4 + 0x31A984])`
fn x87_3(shared: &Shared, s: &Surface, d: &Surface, c: usize) {
    let t0 = ld(s.grip.bits());
    let t1 = ld(d.grip.bits());
    let t2 = t1 - t0;
    let t3 = ld(shared.car.wheel_surface_blend[c].bits());
    let t4 = t2 * t3;
    let t5 = t0 + t4;
    shared.physics_tables.point_grips[c].set_bits(st(t5));
    let t6 = ld(s.wear_factor.bits());
    let t7 = ld(d.wear_factor.bits());
    let t8 = t7 - t6;
    let t9 = ld(shared.car.wheel_surface_blend[c].bits());
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    shared.car.point_wear_factor[c].set_bits(st(t11));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x21411C] = (int [s + k * 4 + 4] + ((int [s + k * 4 + 0x1C] - int [s + k * 4 + 4]) * [rc + 0x31A258])) * [rc + 0x1C5CFC]`
/// `[rc + 0x214120] = (int [d + k * 4 + 4] + ((int [d + k * 4 + 0x1C] - int [d + k * 4 + 4]) * [rc + 0x31A258])) * [rc + 0x1C5CFC]`
fn x87_4(race: &Race, s: &[Dword], d: &[Dword], k: usize) {
    let t6 = int(s[k + 1].get() as i32);
    let t7 = int(s[k + 7].get() as i32);
    let t8 = t7 - t6;
    let t9 = ld(race.shared.car.surface_blend.bits());
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    let t12 = ld(race.course.numbers.hundredth_a.bits());
    let t13 = t11 * t12;
    race.shared.physics_tables.surface_scales[0].set_bits(st(t13));
    let t14 = int(d[k + 1].get() as i32);
    let t15 = int(d[k + 7].get() as i32);
    let t16 = t15 - t14;
    let t17 = ld(race.shared.car.surface_blend.bits());
    let t18 = t16 * t17;
    let t19 = t14 + t18;
    let t20 = ld(race.course.numbers.hundredth_a.bits());
    let t21 = t19 * t20;
    race.shared.physics_tables.surface_scales[1].set_bits(st(t21));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// `[rc + c * 4 + 0x2146C0] = [s + 0x20] + (([d + 0x20] - [s + 0x20]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x2146D4] = [s + 0x24] + (([d + 0x24] - [s + 0x24]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x2146E8] = [s + 0x14] + (([d + 0x14] - [s + 0x14]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x2146FC] = [s + 0x10] + (([d + 0x10] - [s + 0x10]) * [rc + c * 4 + 0x31A984])`
#[cfg(all(test, target_arch = "x86"))]
fn x87_5(race: &Race, s: &Surface, d: &Surface, c: usize) {
    x87_5_grips(race, s, d, c);
    x87_5_blends(race, s, d, c);
}

/// `x87_5`'s first three blends (the point's three grip figures), needed for every point.
fn x87_5_grips(race: &Race, s: &Surface, d: &Surface, c: usize) {
    let t0 = ld(s.slide_grip.bits());
    let t1 = ld(race.shared.physics_tables.surface_scales[0].bits());
    let t2 = t0 * t1;
    let t3 = ld(d.slide_grip.bits());
    let t4 = ld(race.shared.physics_tables.surface_scales[1].bits());
    let t5 = t3 * t4;
    let t6 = t5 - t2;
    let t7 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t8 = t6 * t7;
    let t9 = t2 + t8;
    let t10 = ld(race.shared.physics_tables.spec_surface[2].bits());
    let t11 = t9 * t10;
    race.shared.physics_tables.wheel_loads[c].set_bits(st(t11));
    let t12 = ld(s.accel_grip.bits());
    let t13 = ld(race.shared.physics_tables.surface_scales[0].bits());
    let t14 = t12 * t13;
    let t15 = ld(d.accel_grip.bits());
    let t16 = ld(race.shared.physics_tables.surface_scales[1].bits());
    let t17 = t15 * t16;
    let t18 = t17 - t14;
    let t19 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t20 = t18 * t19;
    let t21 = t14 + t20;
    let t22 = ld(race.shared.physics_tables.spec_surface[0].bits());
    let t23 = t21 * t22;
    race.shared.physics_tables.wheel_grips[c].set_bits(st(t23));
    let t24 = ld(s.brake_grip.bits());
    let t25 = ld(race.shared.physics_tables.surface_scales[0].bits());
    let t26 = t24 * t25;
    let t27 = ld(d.brake_grip.bits());
    let t28 = ld(race.shared.physics_tables.surface_scales[1].bits());
    let t29 = t27 * t28;
    let t30 = t29 - t26;
    let t31 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t32 = t30 * t31;
    let t33 = t26 + t32;
    let t34 = ld(race.shared.physics_tables.spec_surface[1].bits());
    let t35 = t33 * t34;
    race.shared.physics_tables.wheel_grips_b[c].set_bits(st(t35));
}

/// `x87_5`'s last four blends (`surface_blends`: drag, minimum speed, two mud rates), read only
/// for the centre point (4) by `x87_7`; the other points' are not computed.
fn x87_5_blends(race: &Race, s: &Surface, d: &Surface, c: usize) {
    let t36 = ld(s.drag.bits());
    let t37 = ld(d.drag.bits());
    let t38 = t37 - t36;
    let t39 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t40 = t38 * t39;
    let t41 = t36 + t40;
    race.shared.physics_tables.surface_blends[c].set_bits(st(t41));
    let t42 = ld(s.rolling_drag_min_speed.bits());
    let t43 = ld(d.rolling_drag_min_speed.bits());
    let t44 = t43 - t42;
    let t45 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t46 = t44 * t45;
    let t47 = t42 + t46;
    race.shared.physics_tables.surface_blends[5 + c].set_bits(st(t47));
    let t48 = ld(s.mud_rates[1].bits());
    let t49 = ld(d.mud_rates[1].bits());
    let t50 = t49 - t48;
    let t51 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t52 = t50 * t51;
    let t53 = t48 + t52;
    race.shared.physics_tables.surface_blends[10 + c].set_bits(st(t53));
    let t54 = ld(s.mud_rates[0].bits());
    let t55 = ld(d.mud_rates[0].bits());
    let t56 = t55 - t54;
    let t57 = ld(race.shared.car.wheel_surface_blend[c].bits());
    let t58 = t56 * t57;
    let t59 = t54 + t58;
    race.shared.physics_tables.surface_blends[15 + c].set_bits(st(t59));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214268] = ((([rc + 0x214284] + [rc + 0x214288]) + [rc + 0x21428C]) + [rc + 0x214290]) * [rc + 0x213FEC]`
fn x87_6(race: &Race) {
    let t0 = ld(race.shared.physics_tables.wheel_loads[0].bits());
    let t1 = ld(race.shared.physics_tables.wheel_loads[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(race.shared.physics_tables.wheel_loads[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(race.shared.physics_tables.wheel_loads[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(race.shared.physics_tables.wheel_load_scale.bits());
    let t8 = t6 * t7;
    race.shared.physics_tables.wheel_load.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A038] = [rc + 0x31A038] * [rc + 0x214C04]`
/// `[rc + 0x224040] = (([rc + 0x213F5C] - [rc + 0x213F64]) * [rc + 0x31A258]) + [rc + 0x213F64]`
/// `[rc + 0x224044] = (([rc + 0x213F60] - [rc + 0x213F68]) * [rc + 0x31A258]) + [rc + 0x213F68]`
/// `[rc + 0x214230] = (([rc + 0x224044] - [rc + 0x224040]) * (int [rc + 0x31A00C] / [rc + 0x1C5E2C])) + [rc + 0x224040]`
/// `[rc + 0x214760] = [rc + 0x2146D0]`
/// `[rc + 0x214764] = [rc + 0x2146E4]`
/// `[rc + 0x223DA8] = [rc + 0x2146F8]`
/// `[rc + 0x223DAC] = [rc + 0x21470C]`
fn x87_7(race: &Race, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = ld(race.shared.car.ground_bump_scale.bits());
    let t1 = ld(race.shared.physics_tables.handling[6].bits());
    let t2 = t0 * t1;
    race.shared.car.ground_bump_scale.set_bits(st(t2));
    let t3 = ld(race.shared.physics_tables.surface_blend[0].bits());
    let t4 = ld(race.shared.physics_tables.surface_blend[2].bits());
    let t5 = t3 - t4;
    let t6 = ld(race.shared.car.surface_blend.bits());
    let t7 = t5 * t6;
    let t8 = ld(race.shared.physics_tables.surface_blend[2].bits());
    let t9 = t7 + t8;
    drawing.scene.surface_point[0].set_bits(st(t9));
    let t10 = ld(race.shared.physics_tables.surface_blend[1].bits());
    let t11 = ld(race.shared.physics_tables.surface_blend[3].bits());
    let t12 = t10 - t11;
    let t13 = ld(race.shared.car.surface_blend.bits());
    let t14 = t12 * t13;
    let t15 = ld(race.shared.physics_tables.surface_blend[3].bits());
    let t16 = t14 + t15;
    drawing.scene.surface_point[1].set_bits(st(t16));
    let t17 = ld(drawing.scene.surface_point[1].bits());
    let t18 = ld(drawing.scene.surface_point[0].bits());
    let t19 = t17 - t18;
    let t20 = int(race.shared.car.suspension_stiffness.bits() as i32);
    let t21 = ld(race.course.numbers.k256.bits());
    let t22 = t20 / t21;
    let t23 = t19 * t22;
    let t24 = ld(drawing.scene.surface_point[0].bits());
    let t25 = t23 + t24;
    race.shared
        .physics_tables
        .surface_softness
        .set_bits(st(t25));
    let t26 = ld(race.shared.physics_tables.surface_blends[4].bits());
    race.shared.physics_tables.wheel_drag[0].set_bits(st(t26));
    let t27 = ld(race.shared.physics_tables.surface_blends[9].bits());
    race.shared.physics_tables.wheel_drag[1].set_bits(st(t27));
    let t28 = ld(race.shared.physics_tables.surface_blends[14].bits());
    race.shared.vars.wheel_turn_a[0].set_bits(st(t28));
    let t29 = ld(race.shared.physics_tables.surface_blends[19].bits());
    race.shared.vars.wheel_turn_a[1].set_bits(st(t29));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    /// The dwords of a test buffer a run with index `k` reads (`k + 8` of them).
    fn table(base: usize, k: usize) -> &'static [Dword] {
        // SAFETY: the buffer holds the offsets up to `k * 4 + 0x1C` and 16 bytes more,
        // lent for the run and aligned for its cells.
        unsafe { std::slice::from_raw_parts(crate::pointers::host_at(base) as *const Dword, k + 8) }
    }

    // The original's instructions, one function per run.
    reference!(ref_0(t, c, rc) "fld dword ptr [{t} + {c}*4]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x22400C]");
    reference!(ref_1(rc, c) "fild dword ptr [{rc} + 0x224044]", "fmul dword ptr [{rc} + {c}*4 + 0x2D2A14]", "fild dword ptr [{rc} + 0x224048]", "fmul dword ptr [{rc} + {c}*4 + 0x2D2B14]", "fild dword ptr [{rc} + 0x224040]", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fstp dword ptr [{rc} + {c}*4 + 0x31A984]");
    reference!(ref_2(s, d, rc, c) "fld dword ptr [{s} + 0x28]", "fld dword ptr [{d} + 0x28]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2142C0]", "fld dword ptr [{s} + 0x2C]", "fld dword ptr [{d} + 0x2C]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x31A920]");
    reference!(ref_3(s, rc, d, k) "fild dword ptr [{s}]", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x21411C]", "fild dword ptr [{d}]", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x214120]", "fild dword ptr [{s} + {k}*4 + 4]", "fild dword ptr [{s} + {k}*4 + 0x1C]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + 0x31A258]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x21411C]", "fild dword ptr [{d} + {k}*4 + 4]", "fild dword ptr [{d} + {k}*4 + 0x1C]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + 0x31A258]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x214120]");
    reference!(ref_4(s, rc, d, c) "fld dword ptr [{s} + 0x4]", "fmul dword ptr [{rc} + 0x21411C]", "fld dword ptr [{d} + 0x4]", "fmul dword ptr [{rc} + 0x214120]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x214BBC]", "fstp dword ptr [{rc} + {c}*4 + 0x214284]", "fld dword ptr [{s} + 0x8]", "fmul dword ptr [{rc} + 0x21411C]", "fld dword ptr [{d} + 0x8]", "fmul dword ptr [{rc} + 0x214120]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x214BB4]", "fstp dword ptr [{rc} + {c}*4 + 0x214298]", "fld dword ptr [{s} + 0xC]", "fmul dword ptr [{rc} + 0x21411C]", "fld dword ptr [{d} + 0xC]", "fmul dword ptr [{rc} + 0x214120]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x214BB8]", "fstp dword ptr [{rc} + {c}*4 + 0x2142AC]", "fld dword ptr [{s} + 0x20]", "fld dword ptr [{d} + 0x20]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146C0]", "fld dword ptr [{s} + 0x24]", "fld dword ptr [{d} + 0x24]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146D4]", "fld dword ptr [{s} + 0x14]", "fld dword ptr [{d} + 0x14]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146E8]", "fld dword ptr [{s} + 0x10]", "fld dword ptr [{d} + 0x10]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146FC]");
    reference!(ref_5(rc) "fld dword ptr [{rc} + 0x214284]", "fadd dword ptr [{rc} + 0x214288]", "fadd dword ptr [{rc} + 0x21428C]", "fadd dword ptr [{rc} + 0x214290]", "fmul dword ptr [{rc} + 0x213FEC]", "fstp dword ptr [{rc} + 0x214268]");
    reference!(ref_6(rc) "fld dword ptr [{rc} + 0x31A038]", "fmul dword ptr [{rc} + 0x214C04]", "fstp dword ptr [{rc} + 0x31A038]", "fld dword ptr [{rc} + 0x213F5C]", "fsub dword ptr [{rc} + 0x213F64]", "fmul dword ptr [{rc} + 0x31A258]", "fadd dword ptr [{rc} + 0x213F64]", "fstp dword ptr [{rc} + 0x224040]", "fld dword ptr [{rc} + 0x213F60]", "fsub dword ptr [{rc} + 0x213F68]", "fmul dword ptr [{rc} + 0x31A258]", "fadd dword ptr [{rc} + 0x213F68]", "fstp dword ptr [{rc} + 0x224044]", "fld dword ptr [{rc} + 0x224044]", "fsub dword ptr [{rc} + 0x224040]", "fild dword ptr [{rc} + 0x31A00C]", "fdiv dword ptr [{rc} + 0x1C5E2C]", ".byte 0xDE, 0xC9", "fadd dword ptr [{rc} + 0x224040]", "fstp dword ptr [{rc} + 0x214230]", "fld dword ptr [{rc} + 0x2146D0]", "fstp dword ptr [{rc} + 0x214760]", "fld dword ptr [{rc} + 0x2146E4]", "fstp dword ptr [{rc} + 0x214764]", "fld dword ptr [{rc} + 0x2146F8]", "fstp dword ptr [{rc} + 0x223DA8]", "fld dword ptr [{rc} + 0x21470C]", "fstp dword ptr [{rc} + 0x223DAC]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "wheel_surfaces line 68 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Value(0),
                    Operand::RaceBlock {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x22400C, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        &crate::race::owner::race().course,
                        view(a[0], 0),
                        a[1],
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "wheel_surfaces line 68 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(1),
                    Operand::RaceBlock {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x22400C, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        &crate::race::owner::race().course,
                        view(a[0], 0),
                        a[1],
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "wheel_surfaces line 68 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x10, In::F)],
                        writes: &[],
                    },
                    Operand::Value(4),
                    Operand::RaceBlock {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x22400C, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        &crate::race::owner::race().course,
                        view(a[0], 0),
                        a[1],
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "wheel_surfaces line 134 idx c=0",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x224040, In::I),
                            (0x224044, In::I),
                            (0x224048, In::I),
                            (0x2D2A14, In::F),
                            (0x2D2B14, In::F),
                        ],
                        writes: &[(0x31A984, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    x87_2(
                        crate::race::owner::race(),
                        a[1],
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "wheel_surfaces line 134 idx c=1",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x224040, In::I),
                            (0x224044, In::I),
                            (0x224048, In::I),
                            (0x2D2A18, In::F),
                            (0x2D2B18, In::F),
                        ],
                        writes: &[(0x31A988, Out::F)],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    x87_2(
                        crate::race::owner::race(),
                        a[1],
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "wheel_surfaces line 134 idx c=4",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x224040, In::I),
                            (0x224044, In::I),
                            (0x224048, In::I),
                            (0x2D2A24, In::F),
                            (0x2D2B24, In::F),
                        ],
                        writes: &[(0x31A994, Out::F)],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    x87_2(
                        crate::race::owner::race(),
                        a[1],
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "wheel_surfaces line 161 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A984, In::F)],
                        writes: &[(0x2142C0, Out::F), (0x31A920, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_3(&race().shared, view(a[0], 0), view(a[1], 0), a[3]),
            },
            Block {
                name: "wheel_surfaces line 161 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A988, In::F)],
                        writes: &[(0x2142C4, Out::F), (0x31A924, Out::F)],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_3(&race().shared, view(a[0], 0), view(a[1], 0), a[3]),
            },
            Block {
                name: "wheel_surfaces line 161 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A994, In::F)],
                        writes: &[(0x2142D0, Out::F), (0x31A930, Out::F)],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_3(&race().shared, view(a[0], 0), view(a[1], 0), a[3]),
            },
            Block {
                name: "wheel_surfaces line 178 idx k=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (0x1C, In::I)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                        writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (0x1C, In::I)],
                        writes: &[],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    x87_4(race(), table(a[0], a[3]), table(a[2], a[3]), a[3]);
                },
            },
            Block {
                name: "wheel_surfaces line 178 idx k=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (8, In::I), (0x20, In::I)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                        writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (8, In::I), (0x20, In::I)],
                        writes: &[],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    x87_4(race(), table(a[0], a[3]), table(a[2], a[3]), a[3]);
                },
            },
            Block {
                name: "wheel_surfaces line 178 idx k=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (0x14, In::I), (0x2C, In::I)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                        writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (0x14, In::I), (0x2C, In::I)],
                        writes: &[],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    x87_4(race(), table(a[0], a[3]), table(a[2], a[3]), a[3]);
                },
            },
            Block {
                name: "wheel_surfaces line 201 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[
                            (0x21411C, In::F),
                            (0x214120, In::F),
                            (0x214BB4, In::F),
                            (0x214BB8, In::F),
                            (0x214BBC, In::F),
                            (0x31A984, In::F),
                        ],
                        writes: &[
                            (0x214284, Out::F),
                            (0x214298, Out::F),
                            (0x2142AC, Out::F),
                            (0x2146C0, Out::F),
                            (0x2146D4, Out::F),
                            (0x2146E8, Out::F),
                            (0x2146FC, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_5(race(), view(a[0], 0), view(a[2], 0), a[3]),
            },
            Block {
                name: "wheel_surfaces line 201 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[
                            (0x21411C, In::F),
                            (0x214120, In::F),
                            (0x214BB4, In::F),
                            (0x214BB8, In::F),
                            (0x214BBC, In::F),
                            (0x31A988, In::F),
                        ],
                        writes: &[
                            (0x214288, Out::F),
                            (0x21429C, Out::F),
                            (0x2142B0, Out::F),
                            (0x2146C4, Out::F),
                            (0x2146D8, Out::F),
                            (0x2146EC, Out::F),
                            (0x214700, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_5(race(), view(a[0], 0), view(a[2], 0), a[3]),
            },
            Block {
                name: "wheel_surfaces line 201 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[
                            (0x21411C, In::F),
                            (0x214120, In::F),
                            (0x214BB4, In::F),
                            (0x214BB8, In::F),
                            (0x214BBC, In::F),
                            (0x31A994, In::F),
                        ],
                        writes: &[
                            (0x214294, Out::F),
                            (0x2142A8, Out::F),
                            (0x2142BC, Out::F),
                            (0x2146D0, Out::F),
                            (0x2146E4, Out::F),
                            (0x2146F8, Out::F),
                            (0x21470C, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_5(race(), view(a[0], 0), view(a[2], 0), a[3]),
            },
            Block {
                name: "wheel_surfaces line 258",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x213FEC, In::F),
                        (0x214284, In::F),
                        (0x214288, In::F),
                        (0x21428C, In::F),
                        (0x214290, In::F),
                    ],
                    writes: &[(0x214268, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |_| x87_6(race()),
            },
            Block {
                name: "wheel_surfaces line 299",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213F5C, In::F),
                        (0x213F60, In::F),
                        (0x213F64, In::F),
                        (0x213F68, In::F),
                        (0x2146D0, In::F),
                        (0x2146E4, In::F),
                        (0x2146F8, In::F),
                        (0x21470C, In::F),
                        (0x214C04, In::F),
                        (0x224040, In::F),
                        (0x224044, In::F),
                        (0x31A00C, In::I),
                        (0x31A038, In::F),
                        (0x31A258, In::F),
                    ],
                    writes: &[
                        (0x214230, Out::F),
                        (0x214760, Out::F),
                        (0x214764, Out::F),
                        (0x223DA8, Out::F),
                        (0x223DAC, Out::F),
                        (0x224040, Out::F),
                        (0x224044, Out::F),
                        (0x31A038, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0]) },
                plain: |_| x87_7(race(), crate::scene::owner::scene()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
