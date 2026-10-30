//! `scene::place_object_points` (0x45BE05): an object's points and normals into the
//! view, the points onto the screen.
//!
//! In:  eax, ecx only matter for an object with normals and no points (refused)
//! Out: esi = the matrix, ebx = the object's number, ebp = 0 (or the number of normals
//!      if that is not above 0); eax = the number after the last one placed, ecx = the
//!      address after the last one read (both as they came if nothing was placed);
//!      everything else preserved
//!
//! The object `g_objectNumber` has `dword_632370[n]` points and then
//! `dword_632770[n]` normals, numbered from `dword_631F70[n]`, twelve bytes each at
//! `dword_5C6FA4` + number * 12: a point three whole numbers, a normal three floats.
//! Each gets the record 0x1000 + (number & 0xFFF) of `g_pointRecords` (its +0 = the
//! number). A point goes through the matrix `off_5EA40C` points at (`sub_45C18E`, the
//! same arithmetic as `scene::rotate_point`) and onto the screen as
//! `scene::project_point` does it (`sub_45C237`: not if it is nearer than 4096). With
//! `dword_623DD0` not 0 the point is first moved (`sub_45C29B`) towards the one that
//! many bytes further on in the table, by `flt_623DD4` of the way, rounded to whole
//! numbers in `unk_5CCE18`; `dword_62400C` = that fraction. A normal goes through
//! the matrix without its fourth row (`sub_45C1E7`).
//! One caller, `sub_45BDE1`, which takes this way unless `dword_621A5C` (3DNow) or
//! `g_useSse` is set; those two ways (`sub_45BF89`, `sub_45C04E`) are not ported.
//! Refuses an object's number of 0x100 or more, no records, no points' table, no
//! matrix, and normals without points (the original goes on from whatever eax and
//! ecx hold).
//!
//! The arithmetic (the moved point, the rotation, the normals) is the x87's, in plain Rust
//! over `X` (`math/x87_float.rs`), rounding as the control word says; the original's
//! instructions are `asm_reference` in the tests (2026-10-24).

use crate::errors::{BadInput, Refused};
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::matrices_store::MATRICES;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::point_records;
use crate::scene::project_point::{NEAR, rotate, to_screen};
use crate::scene::x87_mem::{fild, fistp, fld, fstp};

// ---- game data ----
const OBJECTS: u32 = 0x100;

// ---- Rust version ----

/// `sub_45C29B`: the point at `from` moved towards the one `by` bytes on, into
/// `unk_5CCE18`.
fn moved(from: usize, by: u32) -> Result<(), BadInput> {
    let other = from.wrapping_add(by as usize);
    let read = |at: usize| crate::scene::object_points::point(at as u32);
    let (a, b) = (read(from)?, read(other)?);
    // The fraction, the two points, the result, the fraction as it is stored.
    let mut w = [
        RACE_VARS.move_fraction.bits(),
        a[0],
        a[1],
        a[2],
        b[0],
        b[1],
        b[2],
        0,
        0,
        0,
        0,
    ];
    moved_at(w.as_mut_ptr().cast());
    // The original stores the three from `point_moved`, y and z on over the first two
    // group masks (`notes/bugs.md`).
    COURSE.point_moved.set(w[7]);
    COURSE.group_masks[0].set(w[8]);
    COURSE.group_masks[1].set(w[9]);
    DRAW.scratch.set_bits(w[10]);
    Ok(())
}

/// The block of `moved`: the fraction, the two points, the result (three whole numbers) and the
/// fraction as it is stored.
fn moved_at(w: *mut u8) {
    let t0 = fld(w, 0x0);
    let t1 = fild(w, 0xC);
    let t2 = fild(w, 0x18);
    let t3 = t2 - t1;
    let t4 = t3 * t0;
    let t5 = t1 + t4;
    let t6 = fild(w, 0x8);
    let t7 = fild(w, 0x14);
    let t8 = t7 - t6;
    let t9 = t8 * t0;
    let t10 = t6 + t9;
    let t11 = fild(w, 0x4);
    let t12 = fild(w, 0x10);
    let t13 = t12 - t11;
    let t14 = t13 * t0;
    let t15 = t11 + t14;
    fistp(w, 0x1C, t15);
    fistp(w, 0x20, t10);
    fistp(w, 0x24, t5);
    fstp(w, 0x28, t0);
}

/// `sub_45C1E7`: a normal's three floats through the matrix's first three rows, into
/// the record's +0x10, +0x14, +0x18.
fn turn(normal: *const u32, matrix: *const u32, record: *mut u32) {
    turn_normal(normal.cast(), matrix.cast(), record.cast())
}

fn turn_normal(p: *const u8, m: *const u8, o: *mut u8) {
    let t0 = fld(p, 0x8);
    let t1 = fld(p, 0x4);
    let t2 = fld(p, 0x0);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x4);
    let t12 = t11 * t2;
    let t13 = fld(m, 0x14);
    let t14 = t13 * t1;
    let t15 = fld(m, 0x24);
    let t16 = t15 * t0;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = fld(m, 0x8);
    let t20 = t2 * t19;
    let t21 = fld(m, 0x18);
    let t22 = t1 * t21;
    let t23 = fld(m, 0x28);
    let t24 = t0 * t23;
    let t25 = t24 + t22;
    let t26 = t20 + t25;
    fstp(o, 0x10, t10);
    fstp(o, 0x14, t18);
    fstp(o, 0x18, t26);
}

/// What the object is made of, read and checked.
struct Object {
    first: u32,
    points: i32,
    normals: i32,
    table: u32,
    matrix: u32,
}

fn object() -> Result<Object, BadInput> {
    let number: u32 = FRAME_SWITCHES.object_number.bits();
    if number >= OBJECTS {
        return Err(BadInput("an object's number past the tables"));
    }
    let object = Object {
        first: crate::scene::object_slots_store::OBJECT_SLOTS.first_point[number as usize].get(),
        points: crate::scene::object_slots_store::OBJECT_SLOTS.point_counts[number as usize].get()
            as i32,
        normals: crate::scene::object_slots_store::OBJECT_SLOTS.normal_counts[number as usize].get()
            as i32,
        table: COURSE_LISTS.object_points.bits(),
        matrix: MATRICES.object_matrix_ptr.bits(),
    };
    if object.points <= 0 && object.normals > 0 {
        return Err(BadInput("an object of normals and no points"));
    }
    if object.points > 0 && (object.table == 0 || !point_records::held() || object.matrix == 0) {
        return Err(BadInput("no points' table, no records or no matrix"));
    }
    Ok(object)
}

/// The object's points placed and projected, its normals turned. What the original
/// leaves in the registers its one caller's callers never read (`draw_object`).
pub fn place_object_points() -> Result<(), Refused> {
    let object = object()?;
    if object.points <= 0 {
        return Ok(());
    }
    // A copy: nothing the loop writes is the matrix (`off_5EA40C`'s, the object's own).
    let matrix: [u32; 16] =
        crate::pointers::GamePtr::new(object.matrix).read("the object's matrix")?;
    let matrix = matrix.as_ptr();
    let move_by: u32 = RACE_VARS.move_by.bits();
    let mut number = object.first;
    let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
    for _ in 0..object.points {
        // Made on a copy of the record (`scene::point_records`), then stored.
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(record)?;
        r[0] = number;
        let point = if move_by != 0 {
            moved(from as usize, move_by)?;
            // x, y, z read on into `group_masks[0]`, `[1]`, as stored.
            [
                COURSE.point_moved.get(),
                COURSE.group_masks[0].get(),
                COURSE.group_masks[1].get(),
            ]
        } else {
            crate::scene::object_points::point(from)?
        };
        rotate(point.as_ptr() as *const i32, matrix, r.as_mut_ptr());
        r[3] = 0;
        r[7] = 0;
        if f32::from_bits(r[6]) >= NEAR {
            let v = [
                DRAW.view_scale.bits(),
                DRAW.view_centre[0].bits(),
                DRAW.view_centre[1].bits(),
                COURSE.flat_depth.bits(),
            ];
            let flat = STAGE_PREFS.flat_depth.bits() != 0;
            to_screen(r.as_mut_ptr(), &v, flat, DRAW.scratch.as_ptr());
        }
        point_records::set(record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    for _ in 0..object.normals.max(0) {
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(record)?;
        r[0] = number;
        let normal = crate::scene::object_points::point(from)?;
        turn(normal.as_ptr(), matrix, r.as_mut_ptr());
        point_records::set(record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    Ok(())
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `moved`.
    unsafe fn moved_at_ref(w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w}]",
                "fild dword ptr [{w} + 12]",
                "fild dword ptr [{w} + 24]",
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fild dword ptr [{w} + 8]",
                "fild dword ptr [{w} + 20]",
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCB", // fmul st, st(3)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fild dword ptr [{w} + 4]",
                "fild dword ptr [{w} + 16]",
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCC", // fmul st, st(4)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 28]",
                "fistp dword ptr [{w} + 32]",
                "fistp dword ptr [{w} + 36]",
                "fstp dword ptr [{w} + 40]",
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `turn`.
    unsafe fn turn_normal_ref(p: *mut u8, m: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{p} + 8]",
                "fld dword ptr [{p} + 4]",
                "fld dword ptr [{p}]",
                "fld dword ptr [{m}]",
                "fmul st(0), st(1)",
                "fld dword ptr [{m} + 0x10]",
                "fmul st(0), st(3)",
                "fld dword ptr [{m} + 0x20]",
                "fmul st(0), st(5)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fxch st(3)",
                "fld dword ptr [{m} + 4]",
                "fmul st(0), st(2)",
                "fld dword ptr [{m} + 0x14]",
                "fmul st(0), st(4)",
                "fld dword ptr [{m} + 0x24]",
                "fmul st(0), st(3)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fxch st(3)",
                "fld dword ptr [{m} + 8]",
                "fmulp st(3), st(0)",
                "fld dword ptr [{m} + 0x18]",
                "fmulp st(1), st(0)",
                "fld dword ptr [{m} + 0x28]",
                "fmulp st(2), st(0)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fxch st(2)",
                "fstp dword ptr [{o} + 0x10]",
                "fstp dword ptr [{o} + 0x14]",
                "fstp dword ptr [{o} + 0x18]",
                p = in(reg) p,
                m = in(reg) m,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn moved_matches_the_assembly() {
        // dword 0 the fraction, 1 .. 4 the point, 4 .. 7 the other, 7 .. 10 the whole numbers,
        // 10 the fraction as stored
        differential::<11>(
            "place_object_points moved",
            8000,
            18,
            [Kind::Exact; 11],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                m[0] = match rng.below(3) {
                    0 => rng.unit().to_bits(),
                    1 => (rng.unit() * 2.0).to_bits(),
                    _ => rng.fl(),
                };
                for d in m.iter_mut().skip(1).take(6) {
                    *d = if rng.below(2) == 0 {
                        rng.int()
                    } else {
                        rng.small() as u32
                    };
                }
            },
            |base| unsafe { moved_at_ref(base) },
            moved_at,
        );
    }

    #[test]
    fn turn_matches_the_assembly() {
        // dwords 0 .. 3 the normal, 4 .. 19 the matrix, 20 .. 28 the record
        differential::<28>(
            "place_object_points turn",
            8000,
            19,
            [Kind::Exact; 28],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(3) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                for d in m.iter_mut().skip(4).take(15) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            |base| unsafe { turn_normal_ref(base, base.add(16), base.add(80)) },
            |base| unsafe { turn_normal(base, base.add(16), base.add(80)) },
        );
    }
}
