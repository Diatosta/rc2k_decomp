//! `effects::init_mark_step` (0x456EA7): the stage loading's "init mark" step: the race's cars
//! (`g_raceCars`) into `dword_5C2FE0`, `dword_5C5640` 0x50, `dword_5C563C` 1, the work car's
//! file number (`race::car_file_number` of `g_carBlockStore`'s kind) into `dword_5C562C`;
//! for each of the `dword_63DB94` cars its record's (`g_carRecords`) places +0x23C (0x50 +
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

use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::view_store::RACE_VIEW;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::race::work_store::RACE_WORK;
/// The Rust version (see the header).
// Converted: `rc2k_core::effects::race::{check_cars, kind_row, car_fields}`
pub fn init_mark_step() -> Result<(), Refused> {
    use rc2k_core::effects::race;
    {
        let count: u32 = RACE_VIEW.players.bits();
        let block = |n: u32| RACE_WORK.car_block(n as usize);
        let record = |n: u32| RACE_WORK.car_record(n as usize);
        race::check_cars(count, |n| block(n) != 0 && record(n) != 0)?;
        VIEWER_PANEL.cars_copy.set_bits(RACE_VIEW.race_cars.bits());
        KIND_TABLES.mark_set_count.set_bits(0x50u32);
        KIND_TABLES.mark_set_flag.set_bits(1u32);
        let file = crate::race::mud::car_file_number(CAR_BLOCKS.cars[0].car_kind.bits());
        KIND_TABLES.mark_car_file_number.set_bits(file);
        for n in 0..count {
            let kind = GamePtr::new(block(n) + 0x24).read::<i32>("a mark's field")?;
            // `dword_5C90D8 ..`: rows of 0x134 (`groups[5]` of the kind's record on).
            let row = (LIGHT_RECORDS.record_addr(0) as usize + 0x6C + race::kind_row(kind)) as u32;
            let bad = crate::effects::Sticky::default();
            let fields = race::car_fields(n, |offset| {
                bad.take(GamePtr::new(row + offset as u32).read::<u32>("a mark's field"))
            });
            bad.done()?;
            for (field, v) in fields {
                GamePtr::new(record(n) + field).write::<u32>(v, "a mark's field")?;
            }
        }
        // The file number is left in eax: `init_marks`' seed.
        crate::effects::init_marks::init_marks(VIEWER_PANEL.settings_text.addr(), file)?;
    }
    Ok(())
}
