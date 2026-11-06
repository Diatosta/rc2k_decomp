//! `race::loadlevel_init`'s "reloadobjects" (IDA, 2026-10-16): the object slots' kinds
//! (the cars' three each), the objects loaded, and each car's collision points: its
//! points (12 bytes: x, y, z, whole numbers) sorted by four quicksorts of the original's
//! own (each step of theirs kept: the left side moves one place a pass, keys compared
//! unsigned), turned to floats (/ 65536) into the car's block, two points put in at 4.

use crate::errors::{BadInput, Refused};
use crate::race::kind_store::KIND_TABLES;
use crate::race::owner::Race;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::trap::car_at;
use crate::race::view_store::RACE_VIEW;
use crate::session::Dword;

const LIFT: u32 = 0x5_0000;
const MOST_SORTED: u32 = 0x3D;
const MOST_CONVERTED: u32 = 0x40;

/// `race::car_kind_10` (0x41F4A1) and `race::car_kind_11` (0x41F485): a car's entry of
/// one of the two tables (the car held to 0 .. 0x21).
pub fn car_table(table: &[Dword; 34], car: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    table[n].get()
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
    // Converted: `rc2k_core::race::quick_sort::quick_sort`, on the points' places
    // (`scene::object_points`).
    use rc2k_core::race::quick_sort::{Order, quick_sort};
    let order = if s.down { Order::Down } else { Order::Up };
    if first >= last {
        return Ok(());
    }
    if !(last - first).is_multiple_of(12) {
        return Err(BadInput("a sort's last point not whole points from its first").into());
    }
    let count = ((last - first) / 12 + 1) as usize;
    let at = |n: usize| first.wrapping_add(12 * n as u32);
    let mut list = (0..count)
        .map(|n| crate::scene::object_points::point(at(n)))
        .collect::<Result<Vec<_>, _>>()?;
    quick_sort(&mut list, (s.key / 4) as usize, order)?;
    for (n, p) in list.into_iter().enumerate() {
        crate::scene::object_points::set_point(at(n), p)?;
    }
    Ok(())
}

/// A text to the log.
fn log(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(&text, newline, 1);
}

/// A number to the log.
fn log_number(value: u32, newline: bool) {
    crate::logfile::log_int::log_int(value as i32, newline, 1);
}

pub fn set_up_car_points(race: &Race, slot: u32, block: u32) -> Result<(), Refused> {
    let slot = 11 + slot as usize;
    {
        let count = crate::scene::owner::scene().objects.slots.point_counts[slot].get();
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
            let first = crate::scene::owner::scene().objects.slots.first_point[slot].get();
            let points = first
                .wrapping_mul(12)
                .wrapping_add(race.course.lists.object_points.get());
            let held = (count as i32).clamp(0, MOST_SORTED as i32) as u32;
            race.physics.contact.car_point_count.set(held);
            for n in 0..held {
                let y = points + n * 12 + 4;
                let lifted = crate::scene::object_points::word(y)?.wrapping_add(LIFT);
                crate::scene::object_points::write(y, &lifted.to_le_bytes())?;
            }
            for (sort, start, n) in [
                (BY_Y, 0u32, held),
                (BY_Z, 0, 4),
                (BY_X_DOWN, 0, 2),
                (BY_X_DOWN, 2, 2),
            ] {
                race.physics.contact.car_point_count.set(n);
                if n >= 2 {
                    let first = points + start * 12;
                    sort_points(sort, first, first + n * 12 - 12)?;
                }
            }
            race.physics.contact.car_point_count.set(held);
            for n in 0..held {
                let y = points + n * 12 + 4;
                let lifted = crate::scene::object_points::word(y)?.wrapping_sub(LIFT);
                crate::scene::object_points::write(y, &lifted.to_le_bytes())?;
            }
            // Converted: `rc2k_core::race::cars::{point_float, open_two_points}`.
            let scale = f32::from_bits(race.course.numbers.k65536.bits());
            let car = car_at(block)?;
            let lists = [&car.probe_point_x, &car.probe_point_y, &car.probe_point_z];
            for n in 0..count {
                let p = points + n * 12;
                for (k, list) in lists.into_iter().enumerate() {
                    // The original goes through `dword_62400C` for each `fild`.
                    let c = crate::scene::object_points::word(p + 4 * k as u32)?;
                    crate::scene::owner::scene().drawing.scene.scratch.set(c);
                    list[n as usize].set_bits(rc2k_core::race::cars::point_float(c, scale));
                }
            }
            race.physics.contact.car_point_count.set(count);
            let mut points = lists.map(crate::session::read_bits);
            rc2k_core::race::cars::open_two_points(&mut points);
            for (l, p) in lists.into_iter().zip(points) {
                crate::session::write_bits(l, p);
            }
            race.physics.contact.car_point_count.set(count + 2);
        }
        log(c"carnum", false);
        log_number(car_at(block)?.car_kind.bits(), false);
        log(c"numcollpts", false);
        log_number(race.physics.contact.car_point_count.get(), true);
        car_at(block)?
            .probe_count
            .set_bits(race.physics.contact.car_point_count.get());
    }
    Ok(())
}

/// `race::set_object_kinds` (0x425BB4): `scene::clear_slot_kinds`, the eight fixed kinds at
/// 0x609294, slots 1 .. 0x3F of `g_slotKinds` from `dword_6091B4`, `dword_5CCF7C` =
/// `dword_621844`; with cars, each car's three kinds (its `known_kind`, `car_kind_11`,
/// `car_kind_10` of its block's +0x24) at slots 0x50 + 0x14 * n, +11, +10; the objects
/// loaded; each car's points set up. Refuses more than the eight cars' blocks.
pub fn set_object_kinds(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    // Converted: `rc2k_core::race::stage_objects::{FIXED_KINDS, DEFAULT_SLOTS, CAR_KINDS,
    // car_slot, check_object_cars}` (each car's slots one step on from the first's, as the
    // original counts edi).
    use rc2k_core::race::stage_objects::{
        CAR_KINDS, CAR_SLOT_STEP, CarKind, DEFAULT_SLOTS, FIRST_CAR_SLOT, FIXED_KINDS, car_slot,
        check_object_cars,
    };
    crate::scene::clear_slot_kinds::clear_slot_kinds();
    crate::session::write_bits(&course.stage_prefs.fixed_kinds, FIXED_KINDS);
    for n in DEFAULT_SLOTS {
        // Slots 56 .. 62 take `fixed_kinds`, just set above: the original's table runs on
        // into it (a slip kept).
        let n_defaults = course.stage_prefs.slot_kind_defaults.len();
        let default = match course.stage_prefs.slot_kind_defaults.get(n as usize) {
            Some(cell) => cell.get(),
            None => course.stage_prefs.fixed_kinds[n as usize - n_defaults].get(),
        };
        crate::scene::owner::scene().objects.tables.slot_kinds[n as usize].set(default);
    }
    // `dword_5CCF7C` = `dword_621844`.
    crate::scene::owner::scene().objects.tables.slot_kinds[64].set(FRAME_SWITCHES.kept_kind.bits());
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
            crate::scene::owner::scene().objects.tables.slot_kinds[(slots + slot) as usize].set(k);
        }
        slots += CAR_SLOT_STEP;
    }
    crate::scene::load_front_end_objects::load_front_end_objects(crate::scene::owner::scene())?;
    let mut slot = FIRST_CAR_SLOT;
    for n in 0..cars {
        let block: u32 = crate::race::work_store::RACE_WORK.car_block(n as usize);
        set_up_car_points(race, slot, block)?;
        slot += CAR_SLOT_STEP;
    }
    Ok(())
}

/// `race::reload_objects` (0x425B2F): "LOAD OBJECTS - START" between two lines of dashes,
/// `set_object_kinds`, then "total points", "total polys" and "facemem used" with their
/// numbers, "LOAD OBJECTS - FINIS" and the dashes.
pub fn reload_objects(race: &Race) -> Result<(), Refused> {
    log(c"-------------------------------------", true);
    log(c"LOAD OBJECTS - START", true);
    set_object_kinds(race)?;
    log(c"total points", false);
    log_number(
        crate::scene::owner::scene()
            .objects
            .tables
            .point_count
            .get(),
        false,
    );
    log(c"total polys", false);
    log_number(
        crate::scene::owner::scene().objects.tables.face_count.get(),
        false,
    );
    log(c"facemem used", false);
    log_number(
        crate::scene::owner::scene()
            .drawing
            .object_work
            .face_cursor
            .bits(),
        true,
    );
    log(c"LOAD OBJECTS - FINIS", true);
    log(c"-------------------------------------", true);
    Ok(())
}
