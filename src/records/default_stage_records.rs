//! `records::default_stage_records` (0x47C7E0): gives every stage of a record table a
//! made-up best time and top speed.
//!
//! In:  edi = the record table
//! Out: every register preserved
//! Nothing if the first record's byte +0x54 is not 0 (it has a name there already).
//! Else for every record with tag 0, up to the table's -1 (`sub_47CE05`, a plain function
//! here), from four random numbers under 0xAA: a driver, whose +0x14 goes to the
//! record's +0x74, +0x18 to +0x78 and name (without a NUL) to +0x54; the second + 1000
//! to +0x30; another driver to +0x9C, +0xA0 and +0x7C; the fourth + 1000 to +0x2C.
//! One caller, `menu::set_up_front_end`, on each of thirteen record tables when
//! `var\ral.dat` was not loaded.
//! Refuses a null table, a table without its end, and a name that does not fit.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::random_below::random_below;
use crate::pointers::GamePtr;
use crate::saves::drivers::Driver;

// ---- game data ----

const RECORD_LEN: usize = 0xA4;
/// More records than any table has.
const MOST_RECORDS: usize = 0x1000;

// ---- Rust version ----

/// How many records the table has before its -1 (the first is not looked at for the
/// end, as in the original).
fn records(table: GamePtr) -> Result<usize, BadInput> {
    // Converted: `rc2k_core::files::records::record_count`.
    let records = table.bytes("a record table")?;
    Ok(rc2k_core::files::records::record_count(
        &records,
        MOST_RECORDS,
    )?)
}

/// `table` the table, a place in the stage tables (`StageTables::place_at`).
pub fn default_stage_records(table: GamePtr) -> Result<(), Refused> {
    if table.is_null() {
        return Err(BadInput("no record table").into());
    }
    if table.offset(0x54).read::<u8>("a record name")? != 0 {
        return Ok(());
    }
    // (the record, the dwords by offset, the two names by offset)
    type Made = (GamePtr, [(u32, u32); 6], [(u32, Vec<u8>); 2]);
    let mut made: Vec<Made> = Vec::new();
    for n in 0..records(table)? {
        let record = table.offset((n * RECORD_LEN) as u32);
        if record.read::<u32>("a record tag")? != 0 {
            continue;
        }
        let first = Driver::number(random_below(0xAA)?)?;
        let time = random_below(0xAA)?.wrapping_add(1000);
        let second = Driver::number(random_below(0xAA)?)?;
        let speed = random_below(0xAA)?.wrapping_add(1000);
        made.push((
            record,
            [
                (0x74, first.nation()),
                (0x78, first.car()),
                (0x30, time),
                (0x9C, second.nation()),
                (0xA0, second.car()),
                (0x2C, speed),
            ],
            [(0x54, first.name()?.1), (0x7C, second.name()?.1)],
        ));
    }
    for (record, dwords, names) in made {
        for (offset, value) in dwords {
            record.offset(offset).write(value, "a stage record")?;
        }
        for (offset, name) in names {
            record.offset(offset).write_bytes(&name, "a stage record")?;
        }
    }
    Ok(())
}
