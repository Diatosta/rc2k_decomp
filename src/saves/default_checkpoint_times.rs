//! `saves::default_checkpoint_times` (0x47CA46): made-up best times with checkpoint times
//! for the championship's stages.
//!
//! In:  nothing
//! Out: every register preserved
//! For each stage of the championship's record table (counted into `seconds`): the ten
//! entries of the stage in the table at `dword_7556E8` get the record's +0x18 + 24,000
//! and 0x9F more each, and a driver, with the record's +4 as the stage for the
//! checkpoints (`saves/default_times.rs`; its caller has `dword_755A0C` at 1 for this
//! one, so the checkpoint times are guessed and some 9,000 pieces go to the game's log).
//! Then the records are gone through once more as in `default_championship_times`, but
//! only `dword_7559B0` is added up and cleared: nothing is done with it.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a record table without stages or without its end. A
//! refusal can come with part of the table written.
//! Self-test: without the checkpoints against the original; with them against a copy
//! whose log calls go to the test (and a copy of `default_time_entry` under it).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::default_checkpoint_times",
    rva(0x47CA46),
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
const TABLE_RVA: usize = rva(0x7556E8); // dword_7556E8

// ---- Rust version ----

/// `ebp` is not used: the stage comes from the record.
pub unsafe fn default_checkpoint_times(_ebp: u32, log: Log) -> Result<(), Refused> {
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
            let time = ((record + 0x18) as *const u32)
                .read_unaligned()
                .wrapping_add(0x5DC0);
            let track = ((record + 4) as *const u32).read_unaligned();
            ten_entries(table + stage * STAGE_LEN, time, 0x9F, track, log)?;
        }
        game_write(WORK_B_RVA, 0u32);
        for n in 0..count as u32 {
            let record = records_at + n * RECORD_LEN;
            match (record as *const u32).read_unaligned() {
                0 => {
                    let length = ((record + 0x18) as *const u32).read_unaligned();
                    let sum = game_read::<u32>(WORK_B_RVA).wrapping_add(length.wrapping_add(0xB6));
                    game_write(WORK_B_RVA, sum);
                }
                3 | 4 => game_write(WORK_B_RVA, 0u32),
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
        Ok(vec![table_region(TABLE_RVA, stages * STAGE_LEN)?])
    }
}

fn run(r: &Regs, log: Log) -> Result<(), Refused> {
    unsafe { default_checkpoint_times(r.ebp, log) }
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
    with_checkpoints(&mut t, &regions, saved.0.len());
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}

const LEN: usize = 0xC4;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x3D, 0x76, 0x55, 0x73, 0x00, 0xE8];
/// `saves::count_stages` and `default_time_entry`.
const CALLS: [usize; 2] = [0x07, 0x5C];
const ENTRY_RVA: usize = rva(0x47CB74);
const ENTRY_FN_LEN: usize = 0x233;
const ENTRY_FN_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0x0C, 0x5A, 0x75, 0x00, 0x01];
const ENTRY_FN_CALLS: [usize; 28] = [
    0x19, 0x26, 0x33, 0x3C, 0x49, 0x50, 0x57, 0x64, 0x70, 0x7C, 0x88, 0x94, 0xA0, 0xAC, 0xB8, 0xC4,
    0xD0, 0xDC, 0xE8, 0xF4, 0x100, 0x10C, 0x118, 0x124, 0x131, 0x13E, 0x149, 0x22C,
];
/// The four log functions it calls: `log_write_l1`, `_l1_nl`, `log_int_l1`, `_l1_nl`.
const LOG_FNS: [usize; 4] = [rva(0x401427), rva(0x40142B), rva(0x401472), rva(0x401476)];

/// What a run logged: each piece's value (a text's address or a number).
static LOGGED: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

fn note_piece(_: u32, r: &mut Regs) {
    LOGGED.lock().unwrap().push(r.eax);
}

/// The case with the checkpoints: a copy of the original over a copy of
/// `default_time_entry` whose log calls only note what they are given.
fn with_checkpoints(t: &mut SelfTest, regions: &[Region], len: usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    use crate::redirect::RedirectedCopy;
    let entry = unsafe {
        RedirectedCopy::new(
            game_addr(ENTRY_RVA),
            ENTRY_FN_LEN,
            &ENTRY_FN_BYTES,
            &ENTRY_FN_CALLS,
            &|callee| LOG_FNS.contains(&callee).then(|| stand_in(0)),
        )
    };
    let Ok(entry) = entry else {
        t.check(false, || "no copy of default_time_entry".to_string());
        return;
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x47CA46)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == ENTRY_RVA).then(|| entry.addr()),
        )
    };
    let Ok(copy) = copy else {
        t.check(false, || "no copy of the function".to_string());
        return;
    };
    for seed in [3u32, 0xFEED_F00D] {
        let before = Regs::SENTINEL;
        let run_side = |rust: bool| unsafe {
            restore(regions, &vec![0xAA; len]);
            game_write(STATE_RVA, seed);
            game_write(WITH_CHECKPOINTS_RVA, 1u32);
            LOGGED.lock().unwrap().clear();
            set_handler(note_piece);
            let ended = if rust {
                run(&before, &mut |piece| {
                    let (Piece::Text(value, _) | Piece::Number(value, _)) = piece;
                    LOGGED.lock().unwrap().push(value);
                })
                .is_err() as u32
            } else {
                let mut ro = before;
                crate::framework::call_game(&mut ro, copy.addr());
                !ro.preserved_except(&before, 0) as u32
            };
            (ended, capture(regions), LOGGED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run_side(false), run_side(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!(
                "with checkpoints, seed {seed:#x}: original ended {} with {} log pieces; rust ended {} with {}; first different piece {:?}; first different byte left {:?}",
                theirs.0,
                theirs.2.len(),
                mine.0,
                mine.2.len(),
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
}
