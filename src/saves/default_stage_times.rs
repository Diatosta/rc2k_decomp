//! `saves::default_stage_times` (0x47C7FE): made-up best times for the stages of the six record tables
//! of `off_73555A`.
//!
//! In:  ebp (passed on as the stage to `default_time_entry`, which only uses it while `dword_755A0C` is 1)
//! Out: every register preserved
//! For each of the six tables and each of its records 1 .. 6: `seconds` (0x7559AC) = the
//! record's +0x18 + 24,000, and the ten entries the record's +0x3C picks in the table at
//! `dword_7556E0` get that and 0x8B more each, and a driver (`saves/default_times.rs`).
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a record that picks entries past the table's 36 stages.
//! Converted: `rc2k_core::saves::defaults::stage_times` (a refusal now leaves nothing written).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::saves::default_times::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, restore};
use rc2k_core::saves::defaults::stage_times;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::default_stage_times",
    rva(0x47C7FE),
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

const TABLE_RVA: usize = rva(0x7556E0); // dword_7556E0
const RECORD_TABLES_RVA: usize = rva(0x73555A); // off_73555A
const TABLES: usize = 6;
/// The stages the table has room for.
const ROOM: u32 = 36;

// ---- Rust version ----

pub unsafe fn default_stage_times(ebp: u32, log: Log) -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::stage_times`.
    unsafe {
        let table: u32 = game_read(TABLE_RVA);
        if table == 0 {
            return Err(BadInput("no table for the times").into());
        }
        let mut tables = Vec::new();
        for n in 0..TABLES {
            let records: u32 = game_read(RECORD_TABLES_RVA + n * 4);
            if records == 0 || records == u32::MAX {
                return Err(BadInput("a record table is missing").into());
            }
            tables.push(std::array::from_fn(|stage| {
                read_record(records + (stage as u32 + 1) * RECORD_LEN)
            }));
        }
        let filled = stage_times(
            &mut GameSource { log },
            &tables,
            Some(ebp),
            with_checkpoints(),
        )?;
        apply(&filled, &[(table, (ROOM * STAGE_LEN) as usize)]);
        Ok(())
    }
}
// ---- hook ----

fn tables() -> Result<Vec<Region>, BadInput> {
    Ok(vec![table_region(TABLE_RVA, ROOM * STAGE_LEN)?])
}

fn run(r: &Regs, log: Log) -> Result<(), Refused> {
    unsafe { default_stage_times(r.ebp, log) }
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
