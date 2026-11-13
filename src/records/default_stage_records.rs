//! Gives every stage of a record table a made-up best time and top speed.
//!
//! Nothing if the first record's holder already has a name. Else every record with tag 0, up
//! to the table's -1, gets two made-up holders from four random numbers under 0xAA: a random
//! driver (nationality, car and name, without a NUL) as the fastest holder with the second
//! number + 1000 as its speed, and another driver as the second holder with the fourth + 1000.
//! Called by `menu::set_up_front_end` on each of thirteen record tables when `var\ral.dat` was
//! not loaded. Refuses a null table, a table without its end, and a name that does not fit.

use crate::errors::{BadInput, Refused};
use crate::math::random_below::random_below;
use crate::menu::owner::Menus;
use crate::records::stages_store::{HOLDER_NAME_LEN, Holder, HolderRow, RecordTable, StageTables};
use crate::saves::drivers::Driver;

/// More records than any table has.
const MOST_RECORDS: usize = 0x1000;

/// How many records the table has before its -1 (the first is not looked at for the end).
fn records(stages: &StageTables, table: RecordTable) -> Result<usize, BadInput> {
    Ok(rc2k_core::files::records::record_count(
        &table.bytes(stages),
        MOST_RECORDS,
    )?)
}

/// A made-up record holder: a random driver's nationality, car and name.
fn holder(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    driver: Driver,
    speed: u32,
) -> Result<(HolderRow, Vec<u8>), Refused> {
    let row = HolderRow {
        nation: driver.nation(),
        car: driver.car(),
        speed,
    };
    Ok((row, driver.name(so, menus)?.1))
}

pub fn default_stage_records(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    stages: &StageTables,
    table: RecordTable,
) -> Result<(), Refused> {
    if table
        .row(stages, 0)?
        .record(stages)?
        .holder_name(Holder::Fastest)[0]
        != 0
    {
        return Ok(());
    }
    let mut made = Vec::new();
    for n in 0..records(stages, table)? {
        let record = table.row(stages, n as u32)?.record(stages)?;
        if record.tag() != 0 {
            continue;
        }
        let first = Driver::of(menus, random_below(0xAA)?)?;
        let time = random_below(0xAA)?.wrapping_add(1000);
        let second = Driver::of(menus, random_below(0xAA)?)?;
        let speed = random_below(0xAA)?.wrapping_add(1000);
        let both = [
            holder(so, menus, first, time)?,
            holder(so, menus, second, speed)?,
        ];
        if both.iter().any(|(_, name)| name.len() > HOLDER_NAME_LEN) {
            return Err(BadInput("a driver's name past a record holder's 0x20 bytes").into());
        }
        made.push((record, both));
    }
    for (record, [(first, first_name), (second, second_name)]) in made {
        record.set_holder(Holder::Fastest, &first, &first_name);
        record.set_holder(Holder::Second, &second, &second_name);
    }
    Ok(())
}
