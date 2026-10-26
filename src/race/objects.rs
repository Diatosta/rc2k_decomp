//! `race::loadlevel_init`'s "reloadobjects" (IDA, 2026-10-16): the object slots' kinds
//! (the cars' three each), the objects loaded, and each car's collision points: its
//! points (12 bytes: x, y, z, whole numbers) sorted by four quicksorts of the original's
//! own (each step of theirs kept: the left side moves one place a pass, keys compared
//! unsigned), turned to floats (/ 65536) into the car's block, two points put in at 4.

use crate::errors::{BadInput, Refused};
use crate::physics::contact_store::CONTACT_WORK;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::kind_store::KIND_TABLES;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::trap::car_at;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::session::Dword;

const LIFT: u32 = 0x5_0000;
const MOST_SORTED: u32 = 0x3D;
const MOST_CONVERTED: u32 = 0x40;
const XS: u32 = 0xAF0;
const YS: u32 = 0xBF0;
const ZS: u32 = 0xCF0;

/// `race::car_kind_10` (0x41F4A1) and `race::car_kind_11` (0x41F485): a car's entry of
/// one of the two tables (the car held to 0 .. 0x21).
pub fn car_table(table: &[Dword; 34], car: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    table[n].get()
}

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a car's point list")
}

fn put(at: u32, value: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(value, "a car's point list")
}

/// One of the four sorts: the key's offset in the point, and whether the order is
/// downwards.
#[derive(Clone, Copy)]
pub struct Sort {
    pub key: u32,
    pub down: bool,
}

pub const BY_Y: Sort = Sort {
    key: 4,
    down: false,
};
pub const BY_Z: Sort = Sort {
    key: 8,
    down: false,
};
pub const BY_X_DOWN: Sort = Sort { key: 0, down: true };

/// The sorts (0x426BE6 by y, 0x426CA2 by z, 0x426D5E and 0x426E16 by x downwards): the
/// points from `first` to `last` (both included), step by step as the original. Refuses
/// (before anything) what `sorting::sort_records` refuses.
pub fn sort_points(s: Sort, first: u32, last: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort` (through `sort_records`).
    use rc2k_core::race::quick_sort::Order;
    let order = if s.down { Order::Down } else { Order::Up };
    crate::race::sorting::sort_records::<3>(first, last, (s.key / 4) as usize, order)
}

/// A text to the log.
fn log(text: &std::ffi::CStr, newline: bool) {
    unsafe { crate::logfile::log_write::log_write(text.as_ptr().cast(), newline, 1) };
}

/// A number to the log.
fn log_number(value: u32, newline: bool) {
    crate::logfile::log_int::log_int(value as i32, newline, 1);
}

/// The variables `set_up_car_points` uses, by address: the game's.
pub struct CarPointVars {
    pub counts: u32,
    pub firsts: u32,
    pub points: u32,
    pub car_points: u32,
    pub scratch: u32,
}

impl CarPointVars {
    pub fn game() -> Self {
        Self {
            counts: crate::scene::object_slots_store::OBJECT_SLOTS.point_counts[11].addr(),
            firsts: crate::scene::object_slots_store::OBJECT_SLOTS.first_point[11].addr(),
            points: COURSE_LISTS.object_points.addr(),
            car_points: CONTACT_WORK.car_point_count.addr(),
            scratch: DRAW.scratch.addr(),
        }
    }
}

/// `race::set_up_car_points` (0x426B67): slot `slot`'s points for the car's block `block` (see
/// the header): with n points, those held to 0 .. 0x3D lifted by 0x50000 in y, sorted by y
/// (all), by z (the first 4), by x downwards (the first 2, and the 2 after), let down
/// again; all n (not held) as floats (each through `dword_62400C`) into the block's +0xAF0 (x), +0xBF0 (y), +0xCF0 (z);
/// moved two on from 4, the new 4 and 5 made (0, the y of 6 at 5's y); `dword_6D31BC` n +
/// 2; logged ("carnum", the car, "numcollpts", the count) and the count into +0x260. With
/// no points only the log and +0x260 (the count as it was). Refuses more than 0x40 points
/// (the original writes past the block's three lists).
pub fn set_up_car_points(slot: u32, block: u32, v: &CarPointVars) -> Result<(), Refused> {
    {
        let count = get(v.counts + 4 * slot)?;
        if count != 0 {
            if count > MOST_CONVERTED {
                return Err(BadInput("more than 0x40 collision points").into());
            }
            if count == 1 {
                // Its first sort's test leaves by a `retn` inside `pusha` (section 16).
                return Err(
                    BadInput("one collision point: the original returns through edi").into(),
                );
            }
            let first = get(v.firsts + 4 * slot)?;
            let points = first.wrapping_mul(12).wrapping_add(get(v.points)?);
            let held = (count as i32).clamp(0, MOST_SORTED as i32) as u32;
            put(v.car_points, held)?;
            for n in 0..held {
                let y = points + n * 12 + 4;
                put(y, get(y)?.wrapping_add(LIFT))?;
            }
            for (sort, start, n) in [
                (BY_Y, 0u32, held),
                (BY_Z, 0, 4),
                (BY_X_DOWN, 0, 2),
                (BY_X_DOWN, 2, 2),
            ] {
                put(v.car_points, n)?;
                if n >= 2 {
                    let first = points + start * 12;
                    sort_points(sort, first, first + n * 12 - 12)?;
                }
            }
            put(v.car_points, held)?;
            for n in 0..held {
                let y = points + n * 12 + 4;
                put(y, get(y)?.wrapping_sub(LIFT))?;
            }
            // Converted: `rc2k_core::race::cars::{point_float, open_two_points}`.
            let scale = f32::from_bits(NUMBERS.k65536.bits());
            for n in 0..count {
                let p = points + n * 12;
                for (k, list) in [XS, YS, ZS].into_iter().enumerate() {
                    // The original goes through `dword_62400C` for each `fild`.
                    put(v.scratch, get(p + 4 * k as u32)?)?;
                    put(
                        block + list + 4 * n,
                        rc2k_core::race::cars::point_float(get(p + 4 * k as u32)?, scale),
                    )?;
                }
            }
            put(v.car_points, count)?;
            let lists = [XS, YS, ZS].map(|list| GamePtr::new(block + list));
            let mut points = [[0u32; 0x40]; 3];
            for (p, l) in points.iter_mut().zip(&lists) {
                *p = l.read("a car's point list")?;
            }
            rc2k_core::race::cars::open_two_points(&mut points);
            for (l, p) in lists.iter().zip(points) {
                l.write(p, "a car's point list")?;
            }
            put(v.car_points, count + 2)?;
        }
        log(c"carnum", false);
        log_number(car_at(block)?.car_kind.bits(), false);
        log(c"numcollpts", false);
        log_number(get(v.car_points)?, true);
        car_at(block)?.probe_count.set_bits(get(v.car_points)?);
    }
    Ok(())
}

/// `race::set_object_kinds` (0x425BB4): `scene::clear_slot_kinds`, the eight fixed kinds at
/// 0x609294, slots 1 .. 0x3F of `g_slotKinds` from `dword_6091B4`, `dword_5CCF7C` =
/// `dword_621844`; with cars, each car's three kinds (its `known_kind`, `car_kind_11`,
/// `car_kind_10` of its block's +0x24) at slots 0x50 + 0x14 * n, +11, +10; the objects
/// loaded; each car's points set up. Refuses more than the eight cars' blocks.
pub fn set_object_kinds() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage_objects::{FIXED_KINDS, DEFAULT_SLOTS, CAR_KINDS,
    // car_slot, check_object_cars}` (each car's slots one step on from the first's, as the
    // original counts edi).
    use rc2k_core::race::stage_objects::{
        CAR_KINDS, CAR_SLOT_STEP, CarKind, DEFAULT_SLOTS, FIRST_CAR_SLOT, FIXED_KINDS, car_slot,
        check_object_cars,
    };
    crate::scene::clear_slot_kinds::clear_slot_kinds();
    crate::session::write_bits(&STAGE_PREFS.fixed_kinds, FIXED_KINDS);
    for n in DEFAULT_SLOTS {
        // Slots 56 .. 62 read past `slot_kind_defaults` into `fixed_kinds`, just set above
        // (the original's table is read on, a slip kept).
        let default: u32 =
            GamePtr::new(STAGE_PREFS.slot_kind_defaults[0].addr() + (4 * n as usize) as u32)
                .read("the slot kind defaults")?;
        OBJECT_TABLES.slot_kinds[n as usize].set(default);
    }
    // `dword_5CCF7C` = `dword_621844`.
    OBJECT_TABLES.slot_kinds[64].set(FRAME_SWITCHES.kept_kind.bits());
    let cars: u32 = RACE_VIEW.race_cars.bits();
    if cars == 0 {
        return Ok(());
    }
    check_object_cars(cars)?;
    let mut slots = car_slot(0);
    for n in 0..cars {
        let block: u32 = crate::race::work_store::RACE_WORK.car_block(n as usize);
        for (kind, slot) in CAR_KINDS {
            let car = car_at(block)?.car_kind.bits();
            let k = match kind {
                CarKind::Known => crate::scene::known_kind::known_kind(car),
                CarKind::Table11 => car_table(&KIND_TABLES.kind_11_table, car),
                CarKind::Table10 => car_table(&KIND_TABLES.kind_10_table, car),
            };
            OBJECT_TABLES.slot_kinds[(slots + slot) as usize].set(k);
        }
        slots += CAR_SLOT_STEP;
    }
    crate::scene::load_front_end_objects::load_front_end_objects()?;
    let mut slot = FIRST_CAR_SLOT;
    for n in 0..cars {
        let block: u32 = crate::race::work_store::RACE_WORK.car_block(n as usize);
        set_up_car_points(slot, block, &CarPointVars::game())?;
        slot += CAR_SLOT_STEP;
    }
    Ok(())
}

/// `race::reload_objects` (0x425B2F): "LOAD OBJECTS - START" between two lines of dashes,
/// `set_object_kinds`, then "total points", "total polys" and "facemem used" with their
/// numbers, "LOAD OBJECTS - FINIS" and the dashes.
pub fn reload_objects() -> Result<(), Refused> {
    log(c"-------------------------------------", true);
    log(c"LOAD OBJECTS - START", true);
    set_object_kinds()?;
    log(c"total points", false);
    log_number(OBJECT_TABLES.point_count.get(), false);
    log(c"total polys", false);
    log_number(OBJECT_TABLES.face_count.get(), false);
    log(c"facemem used", false);
    log_number(OBJECT_WORK.face_cursor.bits(), true);
    log(c"LOAD OBJECTS - FINIS", true);
    log(c"-------------------------------------", true);
    Ok(())
}
