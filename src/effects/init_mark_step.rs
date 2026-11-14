//! `effects::init_mark_step`: the stage loading's "init mark" step: the race's cars go into the
//! effects settings' car list, the own car's object (0x50 but in a network race), a switch 1, and
//! the work car's file number (`race::car_file_number` of the work car's kind); for each of the
//! player-count cars (in a network race every car) its record's places +0x23C (0x50 + 0x14 a car)
//! and +0x240 (0xA more), seven values from its kind's row of the light records (the kind its
//! block's +0x24 held to 0 .. 0x16, rows of 0x134) and the seventeen fixed numbers +0x36C ..
//! +0x3AC; then `effects::init_marks` with the settings.
//!
//! The file number is worked out by `race::car_file_number` and is also `init_marks`' seed. Refuses
//! a car count of 0 or past 8 and a missing block or record, before anything.

use crate::effects::owner::Effects;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::race::light_records_store::LightRecords;
use crate::race::owner::Race;
/// The dword `offset` bytes into the kind's row: the rows start at the sixth group of a record,
/// and the kind is held to the 23 records the rows are laid in, so the seven offsets asked
/// (up to 0x28) fall inside that record's groups.
fn kind_row_word(records: &LightRecords, kind: i32, offset: usize) -> Result<u32, BadInput> {
    use rc2k_core::effects::race;
    const SIXTH_GROUP: usize = 5;
    let record = records
        .records
        .get(race::kind_row(kind) / race::ROW_LEN)
        .ok_or(BadInput("a kind's row past the light records"))?;
    record
        .groups
        .get(SIXTH_GROUP + offset / 4)
        .map(crate::session::PlainDword::get)
        .ok_or(BadInput("a kind's row field past the groups"))
}

// Converted: `rc2k_core::effects::race::{check_cars, kind_row, car_fields}`
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
        let count: u32 = if net.vars.session.get() == 1 {
            race.shared.view.get().race_cars.get()
        } else {
            race.shared.view.get().players.get()
        };
        let records = race.shared.car_records()?;
        // `check_cars` holds the count to 8 before `record` is asked.
        let record = |n: u32| records[n as usize];
        race::check_cars(count, |n| race.shared.work.get().has_car_block(n as usize))?;
        race.effects_settings()
            .cars_copy
            .set(race.shared.view.get().race_cars.get());
        // The one object the effects' shapes are made from: the player's car (the first car,
        // not a joined machine's).
        let own = race.shared.own_car_number(net.vars.session.get()) as u32;
        race.effects_settings().first_object.set(0x50 + 0x14 * own);
        race.effects_settings().object_count.set(1);
        let file = crate::race::mud::car_file_number(
            race.kind_tables(),
            race.shared.own_car(net.vars.session.get()).car_kind.get(),
        );
        race.effects_settings().mark_car_file_number.set(file);
        for n in 0..count {
            let kind = race.shared.race_car_or_end(n as usize).car_kind.get() as i32;
            let bad = crate::effects::Sticky::default();
            let fields = race::car_fields(n, |offset| {
                bad.take(kind_row_word(race.light_records(), kind, offset))
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
        crate::effects::init_marks::init_marks(race, effects, file, scene, gfx, system)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::effects::race::{self, SLOT_KIND_ROWS};

    /// Each field of each kind's row is the dword the byte-offset read gave.
    #[test]
    fn row_words_as_the_old_read() {
        let records = Box::new(LightRecords::new());
        for (k, record) in records.records.iter().enumerate() {
            for (g, cell) in record.groups.iter().enumerate() {
                cell.set(0x1000 * k as u32 + g as u32);
            }
        }
        for kind in -3..40 {
            let row = (0x6C + race::kind_row(kind)) as u32 as usize;
            for offset in SLOT_KIND_ROWS {
                let old = crate::cells::load_value::<u32>(&records.records, row + offset);
                assert_eq!(
                    kind_row_word(&records, kind, offset),
                    old,
                    "{kind} {offset}"
                );
            }
        }
    }
}
