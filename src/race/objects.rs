//! The stage load's "reloadobjects" step: the object slots' kinds (the cars' three each), the
//! objects loaded, and each car's collision points: its points (12 bytes: x, y, z, whole numbers)
//! sorted by four quicksorts (each step kept as is: the left side moves one place a pass, keys
//! compared unsigned), turned to floats (/ 65536) into the car's block, two points put in at 4.

use crate::errors::{BadInput, Refused};
use crate::memory::session::Dword;
use crate::race::owner::Race;

const LIFT: u32 = 0x5_0000;
const MOST_SORTED: u32 = 0x3D;
const MOST_CONVERTED: u32 = 0x40;

/// A car's entry of one of the two kind tables (the car held to 0 .. 0x21).
pub fn car_table(table: &[Dword; 34], car: u32) -> u32 {
    // Logic in `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    table[n].get()
}

/// One of the four sorts: the key's offset in the point, and whether the order is downwards.
#[derive(Clone, Copy)]
pub struct Sort {
    pub key: u32,
    pub down: bool,
}

const BY_Y: Sort = Sort {
    key: 4,
    down: false,
};
const BY_Z: Sort = Sort {
    key: 8,
    down: false,
};
const BY_X_DOWN: Sort = Sort { key: 0, down: true };

/// The sorts (by y, by z, and two by x downwards): the points from `first` to `last` (both
/// included), step by step. Refuses (before anything) what `sorting::sort_records` refuses.
fn sort_points(
    object_points: &crate::scene::object_points_store::ObjectPoints,
    s: Sort,
    first: u32,
    last: u32,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::quick_sort::quick_sort`, on the points' places
    // (`scene::object_points_store`).
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
        .map(|n| object_points.point(at(n)))
        .collect::<Result<Vec<_>, _>>()?;
    quick_sort(&mut list, (s.key / 4) as usize, order)?;
    for (n, p) in list.into_iter().enumerate() {
        object_points.set_point(at(n), p)?;
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

pub fn set_up_car_points(
    race: &Race,
    slot: u32,
    car_number: usize,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let slot = 11 + slot as usize;
    {
        let count = scene.objects.slots.get().point_counts[slot].get();
        if count != 0 {
            if count > MOST_CONVERTED {
                return Err(BadInput("more than 0x40 collision points").into());
            }
            if count == 1 {
                return Err(
                    BadInput("one collision point: the original returns through edi").into(),
                );
            }
            let first = scene.objects.slots.get().first_point[slot].get();
            let points = first
                .wrapping_mul(12)
                .wrapping_add(race.course.lists.get().object_points.get());
            let held = (count as i32).clamp(0, MOST_SORTED as i32) as u32;
            race.physics.contact.get().car_point_count.set(held);
            for n in 0..held {
                let y = points + n * 12 + 4;
                let lifted = race.course.stage.object_points.word(y)?.wrapping_add(LIFT);
                race.course
                    .stage
                    .object_points
                    .write(y, &lifted.to_le_bytes())?;
            }
            for (sort, start, n) in [
                (BY_Y, 0u32, held),
                (BY_Z, 0, 4),
                (BY_X_DOWN, 0, 2),
                (BY_X_DOWN, 2, 2),
            ] {
                race.physics.contact.get().car_point_count.set(n);
                if n >= 2 {
                    let first = points + start * 12;
                    sort_points(
                        &race.course.stage.object_points,
                        sort,
                        first,
                        first + n * 12 - 12,
                    )?;
                }
            }
            race.physics.contact.get().car_point_count.set(held);
            for n in 0..held {
                let y = points + n * 12 + 4;
                let lifted = race.course.stage.object_points.word(y)?.wrapping_sub(LIFT);
                race.course
                    .stage
                    .object_points
                    .write(y, &lifted.to_le_bytes())?;
            }
            // Logic in `rc2k_core::race::cars::{point_float, open_two_points}`.
            let scale = race.course.numbers.get().k65536.get();
            let car = race.shared.race_car(car_number)?;
            let lists = [&car.probe_point_x, &car.probe_point_y, &car.probe_point_z];
            for n in 0..count {
                let p = points + n * 12;
                for (k, list) in lists.into_iter().enumerate() {
                    // Each value goes through `scratch`.
                    let c = race.course.stage.object_points.word(p + 4 * k as u32)?;
                    scene.drawing.scene.get().scratch.set(c);
                    list[n as usize].set(rc2k_core::race::cars::point_float(c, scale));
                }
            }
            race.physics.contact.get().car_point_count.set(count);
            let mut points = lists.map(crate::memory::session::read_bits);
            rc2k_core::race::cars::open_two_points(&mut points);
            for (l, p) in lists.into_iter().zip(points) {
                crate::memory::session::write_bits(l, p);
            }
            race.physics.contact.get().car_point_count.set(count + 2);
        }
        log(c"carnum", false);
        log_number(race.shared.race_car(car_number)?.car_kind.get(), false);
        log(c"numcollpts", false);
        log_number(race.physics.contact.get().car_point_count.get(), true);
        race.shared
            .race_car(car_number)?
            .probe_count
            .set(race.physics.contact.get().car_point_count.get());
    }
    Ok(())
}

/// Sets the object kinds: `scene::clear_slot_kinds`, the eight fixed kinds, slots 1 .. 0x3F of
/// `slot_kinds` from `slot_kind_defaults`, slot 64 = `kept_kind`; with cars, each car's three
/// kinds (its `known_kind`, `kind_11_table` and `kind_10_table`, taken from its block's kind record) at slots
/// 0x50 + 0x14 * n, +11, +10; then loads the objects and sets up each car's points. Refuses more
/// than the eight cars' blocks.
fn set_object_kinds(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    // Logic in `rc2k_core::race::stage_objects::{FIXED_KINDS, DEFAULT_SLOTS, CAR_KINDS, car_slot,
    // check_object_cars}` (each car's slots one step on from the first's).
    use rc2k_core::race::stage_objects::{
        CAR_KINDS, CAR_SLOT_STEP, CarKind, DEFAULT_SLOTS, FIRST_CAR_SLOT, FIXED_KINDS, car_slot,
        check_object_cars,
    };
    crate::scene::clear_slot_kinds::clear_slot_kinds(scene);
    crate::memory::session::write_bits(&course.stage_prefs.get().fixed_kinds, FIXED_KINDS);
    for n in DEFAULT_SLOTS {
        // Slots 56 .. 62 take `fixed_kinds`, just set above: the defaults table runs on into it (a
        // slip kept as is).
        let n_defaults = course.stage_prefs.get().slot_kind_defaults.len();
        let default = match course.stage_prefs.get().slot_kind_defaults.get(n as usize) {
            Some(cell) => cell.get(),
            None => course.stage_prefs.get().fixed_kinds[n as usize - n_defaults].get(),
        };
        scene.texture_levels_mut().slot_kinds[n as usize] = default;
    }

    scene.texture_levels_mut().slot_kinds[64] = race.shared.switches.get().kept_kind.get();
    let cars: u32 = race.shared.view.get().race_cars.get();
    if cars == 0 {
        return Ok(());
    }
    check_object_cars(cars)?;
    let mut slots = car_slot(0);
    for n in 0..cars {
        for (kind, slot) in CAR_KINDS {
            let car = race.shared.race_car(n as usize)?.car_kind.get();
            let k = match kind {
                CarKind::Known => crate::scene::known_kind::known_kind(race.kind_tables(), car),
                CarKind::Table11 => car_table(&race.kind_tables().kind_11_table, car),
                CarKind::Table10 => car_table(&race.kind_tables().kind_10_table, car),
            };
            scene.texture_levels_mut().slot_kinds[(slots + slot) as usize] = k;
        }
        slots += CAR_SLOT_STEP;
    }
    crate::scene::load_front_end_objects::load_front_end_objects(race, scene)?;
    let mut slot = FIRST_CAR_SLOT;
    for n in 0..cars {
        set_up_car_points(race, slot, n as usize, scene)?;
        slot += CAR_SLOT_STEP;
    }
    Ok(())
}

/// Logs "LOAD OBJECTS - START" between two lines of dashes, runs `set_object_kinds`, then logs
/// "total points", "total polys" and "facemem used" with their numbers, "LOAD OBJECTS - FINIS"
/// and the dashes.
pub(super) fn reload_objects(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    log(c"-------------------------------------", true);
    log(c"LOAD OBJECTS - START", true);
    set_object_kinds(race, scene)?;
    log(c"total points", false);
    log_number(scene.objects.tables.get().point_count.get(), false);
    log(c"total polys", false);
    log_number(scene.texture_levels().face_count, false);
    log(c"facemem used", false);
    log_number(scene.object_work().face_cursor, true);
    log(c"LOAD OBJECTS - FINIS", true);
    log(c"-------------------------------------", true);
    Ok(())
}
