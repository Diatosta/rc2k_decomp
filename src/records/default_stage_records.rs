//! `records::default_stage_records` (0x47C7E0): gives every stage of a record table a
//! made-up best time and top speed.
//!
//! Takes the record table. Nothing if the first record's byte +0x54 is not 0 (it has a name there
//! already). Else for every record with tag 0, up to the table's -1 (0x47CE05, a plain function
//! here), from four random numbers under 0xAA: a driver, whose +0x14 goes to the record's +0x74,
//! +0x18 to +0x78 and name (without a NUL) to +0x54; the second + 1000 to +0x30; another driver to
//! +0x9C, +0xA0 and +0x7C; the fourth + 1000 to +0x2C. One caller, `menu::set_up_front_end`, on
//! each of thirteen record tables when `var\ral.dat` was not loaded. Refuses a null table, a table
//! without its end, and a name that does not fit.

use crate::errors::{BadInput, Refused};
use crate::math::random_below::random_below;
use crate::menu::owner::Menus;
use crate::records::stages_store::{HOLDER_NAME_LEN, Holder, HolderRow, RecordTable};
use crate::saves::drivers::Driver;

// ---- game data ----

/// More records than any table has.
const MOST_RECORDS: usize = 0x1000;

/// How many records the table has before its -1 (the first is not looked at for the
/// end, as in the original).
fn records(table: RecordTable) -> Result<usize, BadInput> {
    // Converted: `rc2k_core::files::records::record_count`.
    Ok(rc2k_core::files::records::record_count(
        &table.bytes(),
        MOST_RECORDS,
    )?)
}

/// A made-up record holder: a random driver's nationality, car and name.
fn holder(menus: &Menus, driver: Driver, speed: u32) -> Result<(HolderRow, Vec<u8>), Refused> {
    let row = HolderRow {
        nation: driver.nation(),
        car: driver.car(),
        speed,
    };
    Ok((row, driver.name(menus)?.1))
}

pub fn default_stage_records(menus: &Menus, table: RecordTable) -> Result<(), Refused> {
    if table.row(0)?.record()?.holder_name(Holder::Fastest)[0] != 0 {
        return Ok(());
    }
    let mut made = Vec::new();
    for n in 0..records(table)? {
        let record = table.row(n as u32)?.record()?;
        if record.tag() != 0 {
            continue;
        }
        let first = Driver::of(menus, random_below(0xAA)?)?;
        let time = random_below(0xAA)?.wrapping_add(1000);
        let second = Driver::of(menus, random_below(0xAA)?)?;
        let speed = random_below(0xAA)?.wrapping_add(1000);
        let both = [holder(menus, first, time)?, holder(menus, second, speed)?];
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
