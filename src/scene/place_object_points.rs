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
//! Converted 2026-10-21: the hook's Rust version is `place_object_points_plain`, through
//! `rc2k_core::scene::projection` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code, and the moved point stays the x87's, its rounding following the
//! control word). Shadow mode and the self-test compare the records' floats and
//! `dword_62400C` within `service::floats`' tolerances (the self-test at both test
//! precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBP, EBX, ECX, ESI, Regs, game_addr, game_ptr, game_read, game_write, rva,
};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::{dword, put};
use crate::scene::project_point::{
    CENTRE_X_RVA, CENTRE_Y_RVA, FLAT_DEPTH_RVA, FLAT_RVA, NEAR, RECORDS_RVA, SCALE_RVA, rotate,
    to_screen,
};
use crate::screens::harness::Left;

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const OBJECTS: u32 = 0x100;
const FIRST_RVA: usize = rva(0x631F70); // dword_631F70
const POINT_COUNT_RVA: usize = rva(0x632370); // dword_632370
const NORMAL_COUNT_RVA: usize = rva(0x632770); // dword_632770
const TABLE_RVA: usize = rva(0x5C6FA4); // dword_5C6FA4
const MATRIX_RVA: usize = rva(0x5EA40C); // off_5EA40C
const MOVE_BY_RVA: usize = rva(0x623DD0); // dword_623DD0
const MOVE_FRACTION_RVA: usize = rva(0x623DD4); // flt_623DD4
const MOVED_RVA: usize = rva(0x5CCE18); // unk_5CCE18

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
            game_read::<u32>(MOVE_FRACTION_RVA),
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
            w = in(reg) w.as_mut_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        game_write(MOVED_RVA, [w[7], w[8], w[9]]);
        game_write(CHECKED_RVA, w[10]);
    }
}

/// `sub_45C1E7`: a normal's three floats through the matrix's first three rows, into
/// the record's +0x10, +0x14, +0x18.
unsafe fn turn(normal: *const u32, matrix: *const u32, record: *mut u32) {
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
            p = in(reg) normal,
            m = in(reg) matrix,
            o = in(reg) record,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// What the object is made of, read and checked.
struct Object {
    number: u32,
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
            number,
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

pub unsafe fn place_object_points(entry: &Regs) -> Result<Left, Refused> {
    unsafe {
        let object = object()?;
        let left = Left::nothing()
            .with(ESI, object.matrix)
            .with(EBX, object.number)
            .with(EBP, object.normals.min(0) as u32);
        if object.points <= 0 {
            return Ok(left.with(EAX, entry.eax).with(ECX, entry.ecx));
        }
        let matrix = object.matrix as *const u32;
        let move_by: u32 = game_read(MOVE_BY_RVA);
        let mut number = object.first;
        let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
        for _ in 0..object.points {
            let record = record_of(object.records, number) as usize;
            put(record, number);
            let point = if move_by != 0 {
                moved(from as usize, move_by);
                game_addr(MOVED_RVA)
            } else {
                from as usize
            };
            rotate(point as *const i32, matrix, record as *mut u32);
            put(record + 0xC, 0);
            put(record + 0x1C, 0);
            if f32::from_bits(dword(record + 0x18)) >= NEAR {
                let v = [
                    game_read(SCALE_RVA),
                    game_read(CENTRE_X_RVA),
                    game_read(CENTRE_Y_RVA),
                    game_read(FLAT_DEPTH_RVA),
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
        Ok(left.with(EAX, number).with(ECX, from))
    }
}

/// The converted `place_object_points` (plain floats): the hook's. The x87 version above
/// stays for the Rust callers in unconverted code. The moved point (`sub_45C29B`) stays the
/// x87's: its rounding follows the control word.
// Converted: `rc2k_core::scene::projection::{make_record, turn_normal}`
pub unsafe fn place_object_points_plain(entry: &Regs) -> Result<Left, Refused> {
    use rc2k_core::scene::projection::{make_record, turn_normal};
    unsafe {
        let object = object()?;
        let left = Left::nothing()
            .with(ESI, object.matrix)
            .with(EBX, object.number)
            .with(EBP, object.normals.min(0) as u32);
        if object.points <= 0 {
            return Ok(left.with(EAX, entry.eax).with(ECX, entry.ecx));
        }
        let matrix = (object.matrix as *const [u32; 16])
            .read_unaligned()
            .map(f32::from_bits);
        let move_by: u32 = game_read(MOVE_BY_RVA);
        let mut number = object.first;
        let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
        for _ in 0..object.points {
            let record = record_of(object.records, number) as usize;
            put(record, number);
            let point = if move_by != 0 {
                moved(from as usize, move_by);
                game_addr(MOVED_RVA)
            } else {
                from as usize
            };
            let made = make_record(
                (point as *const [i32; 3]).read_unaligned(),
                &matrix,
                crate::scene::project_point::screen,
                false,
            );
            for (k, v) in made.view.into_iter().enumerate() {
                put(record + 0x10 + 4 * k, v.to_bits());
            }
            put(record + 0xC, made.screen[0].to_bits());
            put(record + 0x1C, made.screen[1].to_bits());
            if let Some(checked) = made.checked {
                game_write(CHECKED_RVA, checked.to_bits());
            }
            from = from.wrapping_add(12);
            number = number.wrapping_add(1);
        }
        for _ in 0..object.normals.max(0) {
            let record = record_of(object.records, number) as usize;
            put(record, number);
            let normal = (from as *const [u32; 3])
                .read_unaligned()
                .map(f32::from_bits);
            for (k, v) in turn_normal(normal, &matrix).into_iter().enumerate() {
                put(record + 0x10 + 4 * k, v.to_bits());
            }
            from = from.wrapping_add(12);
            number = number.wrapping_add(1);
        }
        Ok(left.with(EAX, number).with(ECX, from))
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { place_object_points_plain(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::floats::floats_entry(r, &rust)
}
