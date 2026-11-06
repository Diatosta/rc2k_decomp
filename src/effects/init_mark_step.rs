//! `effects::init_mark_step` (0x456EA7): the stage loading's "init mark" step: the race's cars
//! (`g_raceCars`) into `dword_5C2FE0`, `dword_5C5640` the own car's object (0x50 but in a network race), `dword_5C563C` 1, the work car's
//! file number (`race::car_file_number` of `g_carBlockStore`'s kind) into `dword_5C562C`;
//! for each of the `dword_63DB94` cars (in a network race every car) its record's (`g_carRecords`) places +0x23C (0x50 +
//! 0x14 a car) and +0x240 (0xA more), seven values from its kind's row of `dword_5C90D8`
//! .. (the kind its block's +0x24 held to 0 .. 0x16, rows of 0x134) and the seventeen
//! fixed numbers +0x36C .. +0x3AC; then `effects::init_marks` with the settings
//! `asc_5C2F84`.
//!
//! In:  nothing (the original reads none of its registers; it keeps edi)
//! Out: nothing (no caller reads the registers it leaves: `race::loadlevel_init` reads
//!      none after 0x444670)
//!
//! Caller: `race::loadlevel_init` (0x444670, with the log off round it). The file number is
//! its Rust version, also `init_marks`' seed; then `init_marks`. Refuses a car count of 0
//! or past 8 (the original's loop would run on) and a missing block or record, before
//! anything.

use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
/// The Rust version (see the header).
// Converted: `rc2k_core::effects::race::{check_cars, kind_row, car_fields}`
pub fn init_mark_step(effects: &Effects) -> Result<(), Refused> {
    use rc2k_core::effects::race;
    {
        // In a network race `g_players` is 1 but this machine's car may be any: every car's
        // record is needed, or a joined machine's own has no marks (the original's slip).
        let count: u32 = if crate::net::store::NET.session.get() == 1 {
            RACE_VIEW.race_cars.bits()
        } else {
            RACE_VIEW.players.bits()
        };
        let block = |n: u32| RACE_WORK.car_block(n as usize);
        let records = RACE_WORK.car_records_resolved()?;
        // `check_cars` holds the count to 8 before `record` is asked.
        let record = |n: u32| records[n as usize];
        race::check_cars(count, |n| block(n) != 0)?;
        MARKS_SETTINGS
            .cars_copy
            .set_bits(RACE_VIEW.race_cars.bits());
        // The one object the effects' shapes are made from: the player's car (the original's
        // fixed 0x50 is the first car, not a joined machine's).
        let own = crate::race::blocks_store::own_car_number() as u32;
        MARKS_SETTINGS.first_object.set_bits(0x50 + 0x14 * own);
        MARKS_SETTINGS.object_count.set_bits(1u32);
        let file =
            crate::race::mud::car_file_number(crate::race::blocks_store::own_car().car_kind.bits());
        MARKS_SETTINGS.mark_car_file_number.set_bits(file);
        for n in 0..count {
            let kind = crate::physics::car_store::Car::at(block(n)).car_kind.get() as i32;
            // `dword_5C90D8 ..`: rows of 0x134 (`groups[5]` of the kind's record on).
            // Rows of the kind's record on, run on past it as the original does.
            let row =
                GamePtr::to(&LIGHT_RECORDS.records).offset((0x6C + race::kind_row(kind)) as u32);
            let bad = crate::effects::Sticky::default();
            // Each field is a dword at a byte offset of the row, a run-on as the original.
            let fields = race::car_fields(n, |offset| {
                bad.take(row.offset(offset as u32).read::<u32>("a mark's field"))
            });
            bad.done()?;
            for (field, v) in fields {
                record(n).at(field).write::<u32>(v, "a mark's field")?;
            }
        }
        // The file number is left in eax: `init_marks`' seed.
        crate::effects::init_marks::init_marks(effects, file)?;
    }
    Ok(())
}
