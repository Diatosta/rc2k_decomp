//! `saves::default_championship_times` (0x47C973): made-up best times for the
//! championship's stages and rallies.
//!
//! In:  ebp (passed on as the stage to `default_time_entry`, which only uses it while
//!      `dword_755A0C` is 1)
//! Out: every register preserved
//! For each stage of the championship's record table (its records with tag 0, counted
//! into `seconds`, 0x7559AC): the ten entries of the stage in the table at
//! `dword_7556D8` get the record's +0x24 + 24,000 and 0x9F more each, and a driver
//! (`saves/default_times.rs`). Then, going through the records once more: a stage adds
//! its +0x18 + 0xB6 to `dword_7559B0`; a record with tag 3 or 4 (a rally's end) gives the
//! twenty entries of the next of the six tables of `off_742C98` that sum and 0xB2 more
//! each at +0x18, and a driver (`sub_47CB0A`), and the sum starts again from 0.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses missing tables, a record table without stages or without its end, and more
//! rallies than the six tables. A refusal can come with part of the tables written.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use crate::saves::drivers::put;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::default_championship_times",
    rva(0x47C973),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const TABLE_RVA: usize = rva(0x7556D8); // dword_7556D8
const RALLY_TABLES_RVA: usize = rva(0x742C98); // off_742C98
const RALLIES: usize = 6;
const RALLY_ENTRIES: u32 = 20;

// ---- Rust version ----

pub unsafe fn default_championship_times(ebp: u32, log: Log) -> Result<(), Refused> {
    unsafe {
        let records_at = game_addr(RECORD_TABLE_RVA) as u32;
        let stages = count_stages(records_at as *const u8)?;
        let count = records(records_at)?;
        let table: u32 = game_read(TABLE_RVA);
        if stages == 0 || table == 0 {
            return Err(BadInput("no stages, or no table for their times").into());
        }
        game_write(WORK_A_RVA, stages);
        for stage in 0..stages {
            let record = stage_record(records_at, stage, count)?;
            let time = ((record + 0x24) as *const u32)
                .read_unaligned()
                .wrapping_add(0x5DC0);
            ten_entries(table + stage * STAGE_LEN, time, 0x9F, ebp, log)?;
        }
        let mut rally = 0;
        game_write(WORK_B_RVA, 0u32);
        for n in 0..count as u32 {
            let record = records_at + n * RECORD_LEN;
            match (record as *const u32).read_unaligned() {
                0 => {
                    let length = ((record + 0x18) as *const u32).read_unaligned();
                    let sum = game_read::<u32>(WORK_B_RVA).wrapping_add(length.wrapping_add(0xB6));
                    game_write(WORK_B_RVA, sum);
                }
                3 | 4 => {
                    if rally == RALLIES {
                        return Err(BadInput("more rallies than tables for them").into());
                    }
                    let entries: u32 = game_read(RALLY_TABLES_RVA + rally * 4);
                    if entries == 0 {
                        return Err(BadInput("a rally without its table").into());
                    }
                    let sum: u32 = game_read(WORK_B_RVA);
                    for entry in 0..RALLY_ENTRIES {
                        let (at, time) =
                            (entries + entry * ENTRY_LEN, sum.wrapping_add(entry * 0xB2));
                        put(at, 0x18, time);
                        default_time_entry(at, time, ebp, log)?;
                    }
                    rally += 1;
                    game_write(WORK_B_RVA, 0u32);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn tables() -> Result<Vec<Region>, BadInput> {
    unsafe {
        let stages = count_stages(game_addr(RECORD_TABLE_RVA) as *const u8)?;
        let mut tables = vec![table_region(TABLE_RVA, stages * STAGE_LEN)?];
        for rally in 0..RALLIES {
            let entries: u32 = game_read(RALLY_TABLES_RVA + rally * 4);
            if entries != 0 {
                tables.push((entries, (RALLY_ENTRIES * ENTRY_LEN) as usize));
            }
        }
        Ok(tables)
    }
}

fn run(r: &Regs, log: Log) -> Result<(), Refused> {
    unsafe { default_championship_times(r.ebp, log) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_times(&HOOK, r, call_original, tables(), &run);
}

// ---- self-test ----

/// The game's own tables, filled by both from the same random numbers without the
/// checkpoints (with them the original writes to the log), and put back.
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(tables) = tables() else {
        return t.skip("the tables of best times are not there yet");
    };
    if unsafe { count_stages(game_addr(RECORD_TABLE_RVA) as *const u8) }.is_err() {
        return t.skip("the record table has no end");
    }
    let regions = [tables, shared_regions()].concat();
    let inputs = [at(WITH_CHECKPOINTS_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for seed in [1u32, 0x1234_5678, 0xDEAD_BEEF] {
        check_state(
            &mut t,
            &format!("seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                game_write(STATE_RVA, seed);
                game_write(WITH_CHECKPOINTS_RVA, 0u32);
            },
            &regions,
            &|r| run(r, &mut to_nowhere).map(|()| Left::nothing()),
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
