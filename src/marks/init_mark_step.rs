//! `marks::init_mark_step` (0x456EA7): the stage loading's "init mark" step: the race's cars
//! (`g_raceCars`) into `dword_5C2FE0`, `dword_5C5640` 0x50, `dword_5C563C` 1, the work car's
//! file number (`race::car_file_number` of `g_carBlockStore`'s kind) into `dword_5C562C`;
//! for each of the `dword_63DB94` cars its record's (`g_carRecords`) places +0x23C (0x50 +
//! 0x14 a car) and +0x240 (0xA more), seven values from its kind's row of `dword_5C90D8`
//! .. (the kind its block's +0x24 held to 0 .. 0x16, rows of 0x134) and the seventeen
//! fixed numbers +0x36C .. +0x3AC; then `marks::init_marks` with the settings
//! `asc_5C2F84`.
//!
//! In:  nothing (the original reads none of its registers; it keeps edi)
//! Out: nothing (no caller reads the registers it leaves: `race::loadlevel_init` reads
//!      none after 0x444670)
//!
//! Caller: `race::loadlevel_init` (0x444670, with the log off round it). The file number
//! is its Rust version, also `init_marks`' seed; then `init_marks`. Refuses a car count of 0 or past 8 (the original's loop would run on) and a
//! missing block or record, before anything.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};

const RACE_CARS_RVA: usize = rva(0x63DB90); // g_raceCars
const CARS_COPY_RVA: usize = rva(0x5C2FE0); // dword_5C2FE0
const SET_RVA: [usize; 2] = [rva(0x5C5640), rva(0x5C563C)];
const KIND_RVA: usize = rva(0x70F410); // g_carBlockStore.kind
const FILE_RVA: usize = rva(0x5C562C); // dword_5C562C
const COUNT_RVA: usize = rva(0x63DB94); // dword_63DB94
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const RECORDS_RVA: usize = rva(0x71B3D8); // g_carRecords
const ROWS_RVA: usize = rva(0x5C90D8); // dword_5C90D8 ..: rows of 0x134
const SETTINGS_RVA: usize = rva(0x5C2F84); // asc_5C2F84
/// The Rust version (see the header).
// Converted: `rc2k_core::marks::race::{check_cars, kind_row, car_fields}`
pub unsafe fn init_mark_step() -> Result<(), Refused> {
    use rc2k_core::marks::race;
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        let block = |n: u32| game_read::<u32>(BLOCKS_RVA + 4 * n as usize);
        let record = |n: u32| game_read::<u32>(RECORDS_RVA + 4 * n as usize);
        race::check_cars(count, |n| block(n) != 0 && record(n) != 0)?;
        game_write(CARS_COPY_RVA, game_read::<u32>(RACE_CARS_RVA));
        game_write(SET_RVA[0], 0x50u32);
        game_write(SET_RVA[1], 1u32);
        let file = crate::race::mud::car_file_number(game_read(KIND_RVA));
        game_write(FILE_RVA, file);
        for n in 0..count {
            let kind = ((block(n) + 0x24) as *const i32).read_unaligned();
            let row = ROWS_RVA + race::kind_row(kind);
            for (field, v) in race::car_fields(n, |offset| game_read(row + offset)) {
                ((record(n) + field) as *mut u32).write_unaligned(v);
            }
        }
        // The file number is left in eax: `init_marks`' seed.
        crate::marks::init_marks::init_marks(game_addr(SETTINGS_RVA) as u32, file)?;
    }
    Ok(())
}
