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
use crate::framework::{game_ptr, game_read, rva};
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::draw_store::DRAW;
use crate::scene::polygon::{dword, put};
use crate::scene::project_point::{CENTRE_Y_RVA, FLAT_RVA, NEAR, RECORDS_RVA, rotate, to_screen};
use crate::scene::x87_mem::{fild, fistp, fld, fstp};

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const OBJECTS: u32 = 0x100;
const FIRST_RVA: usize = rva(0x631F70); // dword_631F70
const POINT_COUNT_RVA: usize = rva(0x632370); // dword_632370
const NORMAL_COUNT_RVA: usize = rva(0x632770); // dword_632770
const TABLE_RVA: usize = rva(0x5C6FA4); // dword_5C6FA4
const MATRIX_RVA: usize = rva(0x5EA40C); // off_5EA40C

// ---- Rust version ----

/// `sub_45C29B`: the point at `from` moved towards the one `by` bytes on, into
/// `unk_5CCE18`.
unsafe fn moved(from: usize, by: u32) {
    unsafe {
        let other = from.wrapping_add(by as usize);
        let read = |at: usize| (at as *const [u32; 3]).read_unaligned();
        let (a, b) = (read(from), read(other));
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
    }
}

/// The block of `moved`: the fraction, the two points, the result (three whole numbers) and the
/// fraction as it is stored.
unsafe fn moved_at(w: *mut u8) {
    unsafe {
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
}

/// `sub_45C1E7`: a normal's three floats through the matrix's first three rows, into
/// the record's +0x10, +0x14, +0x18.
unsafe fn turn(normal: *const u32, matrix: *const u32, record: *mut u32) {
    unsafe { turn_normal(normal.cast(), matrix.cast(), record.cast()) }
}

unsafe fn turn_normal(p: *const u8, m: *const u8, o: *mut u8) {
    unsafe {
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
}

/// What the object is made of, read and checked.
struct Object {
    first: u32,
    points: i32,
    normals: i32,
    table: u32,
    records: u32,
    matrix: u32,
}

unsafe fn object() -> Result<Object, BadInput> {
    unsafe {
        let number: u32 = game_read(OBJECT_RVA);
        if number >= OBJECTS {
            return Err(BadInput("an object's number past the tables"));
        }
        let at = number as usize * 4;
        let object = Object {
            first: game_read(FIRST_RVA + at),
            points: game_read(POINT_COUNT_RVA + at),
            normals: game_read(NORMAL_COUNT_RVA + at),
            table: game_read(TABLE_RVA),
            records: game_read(RECORDS_RVA),
            matrix: game_read(MATRIX_RVA),
        };
        if object.points <= 0 && object.normals > 0 {
            return Err(BadInput("an object of normals and no points"));
        }
        if object.points > 0 && (object.table == 0 || object.records == 0 || object.matrix == 0) {
            return Err(BadInput("no points' table, no records or no matrix"));
        }
        Ok(object)
    }
}

fn record_of(records: u32, number: u32) -> u32 {
    records.wrapping_add(((number & 0xFFF) + 0x1000) << 5)
}

/// The object's points placed and projected, its normals turned. What the original
/// leaves in the registers its one caller's callers never read (`draw_object`).
pub unsafe fn place_object_points() -> Result<(), Refused> {
    unsafe {
        let object = object()?;
        if object.points <= 0 {
            return Ok(());
        }
        let matrix = object.matrix as *const u32;
        let move_by: u32 = RACE_VARS.move_by.bits();
        let mut number = object.first;
        let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
        for _ in 0..object.points {
            let record = record_of(object.records, number) as usize;
            put(record, number);
            let point = if move_by != 0 {
                moved(from as usize, move_by);
                // x, y, z read on into `group_masks[0]`, `[1]`, as stored.
                COURSE.point_moved.addr()
            } else {
                from as usize
            };
            rotate(point as *const i32, matrix, record as *mut u32);
            put(record + 0xC, 0);
            put(record + 0x1C, 0);
            if f32::from_bits(dword(record + 0x18)) >= NEAR {
                let v = [
                    DRAW.view_scale.bits(),
                    DRAW.view_centre[0].bits(),
                    game_read(CENTRE_Y_RVA),
                    COURSE.flat_depth.bits(),
                ];
                let flat = game_read::<u32>(FLAT_RVA) != 0;
                to_screen(record as *mut u32, &v, flat, game_ptr(CHECKED_RVA));
            }
            from = from.wrapping_add(12);
            number = number.wrapping_add(1);
        }
        for _ in 0..object.normals.max(0) {
            let record = record_of(object.records, number) as usize;
            put(record, number);
            turn(from as usize as *const u32, matrix, record as *mut u32);
            from = from.wrapping_add(12);
            number = number.wrapping_add(1);
        }
        Ok(())
    }
}

#[cfg(test)]
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
            |base| unsafe { moved_at(base) },
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
