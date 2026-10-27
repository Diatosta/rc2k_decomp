//! `race::load_stage`'s "race init" (`sub_4462B9`, IDA 2026-10-17): after the best line,
//! the camera's start (`sub_446463`), the face lights, the cars (`sub_43603C`), each car's
//! race state, the race slots, the cars' starts and randoms, then fifty settling steps
//! (`sub_4608BB`, `sub_456484`, the marks) and `sub_43FDFE`. Here its small steps.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::race::course_store::COURSE;
use crate::race::overlay_store::RACE_OVERLAY;
use crate::race::trap::Trap;
use crate::race::trap::car_at;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;

/// The car blocks `g_carBlocks` lists (all eight, whatever the race's count).
pub const BLOCKS: u32 = 8;
/// Car block n (`g_carBlocks`[n]).
pub fn block(n: u32) -> u32 {
    crate::race::work_store::RACE_WORK.car_block(n as usize)
}

fn put(at: u32, value: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(value, "a race variable")
}

/// `race::clear_race_slots` (0x4621CD): the eight `g_raceSlots` records' first dword 0.
pub fn clear_race_slots() {
    for slot in &RACE_WORK.race_slots {
        slot.part.set(0);
    }
}

/// `race::reset_car_starts` (0x444BBC): each car block's +0x14 and +8 0 (0x320 while
/// `dword_71C3E0` is set), +0x18 and +0x1C 0.
pub fn reset_car_starts() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::start_value`.
    let value = rc2k_core::race::cars::start_value(RACE_WORK.in_credits.bits() != 0);
    for n in 0..BLOCKS {
        let b = block(n);
        {
            car_at(b)?.run_ticks.set_bits(value);
            put(b + 8, value)?;
            car_at(b)?.start_in_gear.set_bits(0);
            car_at(b)?.false_start_count.set_bits(0);
        }
    }
    Ok(())
}

/// `race::seed_car_randoms` (0x444BF6): each car block's +0x10 and +0xC one
/// `nonzero_random`.
pub fn seed_car_randoms() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::car_seeds`.
    use crate::math::next_random::{DRAWS, wait_for_random, with_random};
    wait_for_random();
    DRAWS.fetch_add(BLOCKS, std::sync::atomic::Ordering::SeqCst);
    let seeds = with_random(rc2k_core::race::cars::car_seeds);
    for (n, &seed) in seeds.iter().enumerate() {
        let b = block(n as u32);
        {
            car_at(b)?.car_flags.set_bits(seed);
            car_at(b)?.noise_word.set_bits(seed);
        }
    }
    Ok(())
}

/// `race::reset_car_race_state` (0x444C1D): each car block's +0x71C 0xCA, +0x72C 0x12C,
/// +0x720/+0x724/+0x728/+0x730 0, +0xA0C .. +0xA1C -1; the four text dwords at 0x6D1838 ..
/// spaces, the four at 0x6D1850 .. -1.
pub fn reset_car_race_state() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::{RACE_STATE, BLANK_TEXT, UNSET_TEXT}`.
    use rc2k_core::race::cars::{BLANK_TEXT, RACE_STATE, UNSET_TEXT};
    for n in 0..BLOCKS {
        let b = block(n);
        for (off, v) in RACE_STATE {
            put(b + off, v)?;
        }
    }
    // dword_6D1838, a64, dword_6D1844, a32: four dwords of spaces.
    for cell in [
        &RACE_OVERLAY.label_behind[0],
        &RACE_OVERLAY.label_behind[1],
        &RACE_OVERLAY.label_ahead[0],
        &RACE_OVERLAY.label_ahead[1],
    ] {
        cell.set(BLANK_TEXT);
    }
    // dword_6D1850, a64_0, dword_6D185C, a32_0: four dwords of -1.
    for cell in [
        &RACE_OVERLAY.kept_behind[0],
        &RACE_OVERLAY.kept_behind[1],
        &RACE_OVERLAY.kept_ahead[0],
        &RACE_OVERLAY.kept_ahead[1],
    ] {
        cell.set(UNSET_TEXT);
    }
    Ok(())
}

/// `race::first_check_checkpoint` (0x446280): the first checkpoint from 1 whose face's
/// +0x26 is not 0, looked for up to `g_lastCheckpoint` (not included); 0 for none or with
/// `g_checkpointCount` - 2 negative. Refuses a look that would run past the list (the
/// original runs on until it finds one: `g_lastCheckpoint` 0 or 1).
pub fn first_check_checkpoint() -> Result<u32, Refused> {
    // Converted: `rc2k_core::race::checkpoints::first_check_checkpoint`.
    let count: u32 = RACE_VARS.checkpoint_count.bits();
    let last = || RACE_VARS.last_checkpoint.bits();
    let trap = Trap::new();
    let look = rc2k_core::race::checkpoints::first_check_checkpoint(count, last, |n| {
        let face: u32 = crate::race::checkpoints_store::face(n as usize).get();
        let address = (face << 6).wrapping_add(COURSE.course_faces.bits());
        trap.course::<u8>(address + 0x26, "a checkpoint face's check byte")
    })?;
    trap.finish()?;
    Ok(look.and_then(|l| l.found).unwrap_or(0))
}
