//! `effects::init_mark_step`: the stage loading's "init mark" step: the race's cars go into the
//! effects settings' car list, the own car's object (0x50 but in a network race), a switch 1, and
//! the work car's file number (`race::car_file_number` of the work car's kind); for each of the
//! player-count cars (in a network race every car) its record's `object` (0x50 + 0x14 a car) and
//! `second_object` (0xA more), the slot kinds from its kind's row of the light records (the kind of
//! its block held to 0 .. 0x16) and the seventeen fixed `kinds`; then `effects::init_marks` with
//! the settings.
//!
//! The file number is worked out by `race::car_file_number` and is also `init_marks`' seed. Refuses
//! a car count of 0 or past 8 and a missing block or record, before anything.

use crate::effects::owner::Effects;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::race::light_records_store::LightRecordsRef;
use crate::race::owner::Race;
/// Group `group` of the kind's row: the rows start at the sixth group of a record, and the kind
/// is held to the 23 records the rows are laid in, so the groups asked (up to the tenth) fall
/// inside that record's groups.
fn kind_row_word(records: LightRecordsRef<'_>, kind: i32, group: usize) -> Result<u32, BadInput> {
    use rc2k_core::effects::race;
    const SIXTH_GROUP: usize = 5;
    records.read(|records| {
        let record = records
            .records
            .get(race::kind_row(kind) / race::ROW_LEN)
            .ok_or(BadInput("a kind's row past the light records"))?;
        record
            .groups
            .get(SIXTH_GROUP + group)
            .copied()
            .ok_or(BadInput("a kind's row field past the groups"))
    })
}

// The logic is in `rc2k_core::effects::race::{check_cars, kind_row, car_fields}`
pub fn init_mark_step(
    race: &'static Race,
    effects: &Effects,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    use rc2k_core::effects::race;
    {
        // In a network race the session's car count is 1 but this machine's car may be any:
        // every car's record is needed, or a joined machine's own has no marks.
        let count: u32 = if net.race_slot.is_on() {
            race.shared.view.get().race_cars.get()
        } else {
            race.shared.view.get().players.get()
        };
        crate::logfile::load_times::mark("car fields");
        let records = race.car_records()?;
        // `check_cars` holds the count to 8 before `record` is asked.
        let record = |n: u32| records[n as usize];
        race::check_cars(count, |n| race.shared.work().has_car_block(n as usize))?;
        race.effects_settings()
            .cars_copy
            .set(race.shared.view.get().race_cars.get());
        // The one object the effects' shapes are made from: the player's car (the first car,
        // not a joined machine's).
        let own = race.shared.own_car_number(net.race_slot.get()) as u32;
        race.effects_settings().first_object.set(0x50 + 0x14 * own);
        race.effects_settings().object_count.set(1);
        let file = crate::race::mud::car_file_number(
            race.kind_tables(),
            race.shared.own_car(net.race_slot.get()).car_kind.get(),
        );
        race.effects_settings().mark_car_file_number.set(file);
        for n in 0..count {
            let kind = race.shared.race_car_or_end(n as usize).car_kind.get() as i32;
            let bad = crate::effects::Sticky::default();
            let fields = race::car_fields(n, |group| {
                bad.take(kind_row_word(race.light_records(), kind, group))
            });
            bad.done()?;
            let car = record(n);
            car.object.set(fields.object);
            car.second_object.set(fields.second_object);
            for (cell, v) in car.slot_kinds.iter().zip(fields.slot_kinds) {
                cell.set(v);
            }
            for (cell, v) in car.kinds.iter().zip(fields.kinds) {
                cell.set(v);
            }
        }
        // The file number is also `init_marks`' seed.
        crate::logfile::load_times::mark("init_marks");
        crate::effects::init_marks::init_marks(race, effects, file, scene, gfx, system)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::light_records_store::LightRecords;
    use rc2k_core::effects::race::{self, SLOT_KIND_GROUPS};

    /// Each field of each kind's row is the dword the row's byte offset names.
    #[test]
    fn row_words_as_the_old_read() {
        let mut records = Box::new(LightRecords::new());
        for (k, record) in records.records.iter_mut().enumerate() {
            for (g, word) in record.groups.iter_mut().enumerate() {
                *word = 0x1000 * k as u32 + g as u32;
            }
        }
        // The records' words in their layout: head, axles, groups, lights.
        let laid: Vec<u8> = records
            .records
            .iter()
            .flat_map(|r| {
                r.head
                    .iter()
                    .chain(&r.axles)
                    .chain(&r.groups)
                    .chain(&r.lights)
            })
            .flat_map(|word| word.to_le_bytes())
            .collect();
        for kind in -3..40 {
            let row = (0x6C + race::kind_row(kind)) as u32 as usize;
            for group in SLOT_KIND_GROUPS {
                let offset = 4 * group;
                let old = laid
                    .get(row + offset..)
                    .and_then(|rest| rest.first_chunk::<4>())
                    .map(|bytes| u32::from_le_bytes(*bytes))
                    .ok_or(crate::errors::BadInput("past the end of a record"));
                assert_eq!(
                    kind_row_word(LightRecordsRef::Copy(&records), kind, group),
                    old,
                    "{kind} {offset}"
                );
            }
        }
    }
}
