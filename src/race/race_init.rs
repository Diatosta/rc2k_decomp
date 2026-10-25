//! `race::load_stage`'s "race init" (`sub_4462B9`, IDA 2026-10-17): after the best line,
//! the camera's start (`sub_446463`), the face lights, the cars (`sub_43603C`), each car's
//! race state, the race slots, the cars' starts and randoms, then fifty settling steps
//! (`sub_4608BB`, `sub_456484`, the marks) and `sub_43FDFE`. Here its small steps.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::physics::car_store::Car;
use crate::race::best_line::CHECK_FACES_RVA;
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;

pub const SLOTS_RVA: usize = rva(0x71BAC8); // g_raceSlots: 8 records of 0x78, +0 in use
pub const SLOT_SIZE: u32 = 0x78;
pub const SLOTS: u32 = 8;
/// The car blocks `g_carBlocks` lists (all eight, whatever the race's count).
pub const BLOCKS: u32 = 8;
/// dword_6D1838, a64, dword_6D1844, a32: four dwords of spaces.
pub const BLANK_TEXT_RVAS: [usize; 4] =
    [rva(0x6D1838), rva(0x6D183C), rva(0x6D1844), rva(0x6D1848)];
/// dword_6D1850, a64_0, dword_6D185C, a32_0: four dwords of -1.
pub const UNSET_TEXT_RVAS: [usize; 4] =
    [rva(0x6D1850), rva(0x6D1854), rva(0x6D185C), rva(0x6D1860)];

/// Car block n (`g_carBlocks`[n]).
pub fn block(n: u32) -> u32 {
    crate::race::work_store::RACE_WORK.car_block(n as usize)
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// `race::clear_race_slots` (0x4621CD): the eight `g_raceSlots` records' first dword 0.
pub unsafe fn clear_race_slots() {
    let first = game_addr(SLOTS_RVA) as u32;
    for n in 0..SLOTS {
        unsafe { put(first + n * SLOT_SIZE, 0) };
    }
}

/// `race::reset_car_starts` (0x444BBC): each car block's +0x14 and +8 0 (0x320 while
/// `dword_71C3E0` is set), +0x18 and +0x1C 0.
pub unsafe fn reset_car_starts() {
    // Converted: `rc2k_core::race::cars::start_value`.
    let value = rc2k_core::race::cars::start_value(RACE_WORK.in_credits.bits() != 0);
    for n in 0..BLOCKS {
        let b = block(n);
        unsafe {
            Car::at(b).run_ticks.set_bits(value);
            put(b + 8, value);
            Car::at(b).start_in_gear.set_bits(0);
            Car::at(b).false_start_count.set_bits(0);
        }
    }
}

/// `race::seed_car_randoms` (0x444BF6): each car block's +0x10 and +0xC one
/// `nonzero_random`.
pub unsafe fn seed_car_randoms() {
    // Converted: `rc2k_core::race::cars::car_seeds`.
    use crate::math::next_random::{DRAWS, wait_for_random, with_random};
    wait_for_random();
    DRAWS.fetch_add(BLOCKS, std::sync::atomic::Ordering::SeqCst);
    let seeds = with_random(rc2k_core::race::cars::car_seeds);
    for (n, &seed) in seeds.iter().enumerate() {
        let b = block(n as u32);
        unsafe {
            Car::at(b).car_flags.set_bits(seed);
            Car::at(b).noise_word.set_bits(seed);
        }
    }
}

/// `race::reset_car_race_state` (0x444C1D): each car block's +0x71C 0xCA, +0x72C 0x12C,
/// +0x720/+0x724/+0x728/+0x730 0, +0xA0C .. +0xA1C -1; the four text dwords at 0x6D1838 ..
/// spaces, the four at 0x6D1850 .. -1.
pub unsafe fn reset_car_race_state() {
    // Converted: `rc2k_core::race::cars::{RACE_STATE, BLANK_TEXT, UNSET_TEXT}`.
    use rc2k_core::race::cars::{BLANK_TEXT, RACE_STATE, UNSET_TEXT};
    for n in 0..BLOCKS {
        let b = block(n);
        for (off, v) in RACE_STATE {
            unsafe { put(b + off, v) };
        }
    }
    unsafe {
        for at in BLANK_TEXT_RVAS {
            game_write(at, BLANK_TEXT);
        }
        for at in UNSET_TEXT_RVAS {
            game_write(at, UNSET_TEXT);
        }
    }
}

/// `race::first_check_checkpoint` (0x446280): the first checkpoint from 1 whose face's
/// +0x26 is not 0, looked for up to `g_lastCheckpoint` (not included); 0 for none or with
/// `g_checkpointCount` - 2 negative. Refuses a look that would run past the list (the
/// original runs on until it finds one: `g_lastCheckpoint` 0 or 1).
pub unsafe fn first_check_checkpoint() -> Result<u32, Refused> {
    // Converted: `rc2k_core::race::checkpoints::first_check_checkpoint`.
    let count: u32 = RACE_VARS.checkpoint_count.bits();
    let last = || RACE_VARS.last_checkpoint.bits();
    let look = rc2k_core::race::checkpoints::first_check_checkpoint(count, last, |n| {
        let face: u32 = unsafe { game_read(CHECK_FACES_RVA + 4 * n as usize) };
        let address = (face << 6).wrapping_add(COURSE.course_faces.bits());
        unsafe { *((address + 0x26) as *const u8) }
    })?;
    Ok(look.and_then(|l| l.found).unwrap_or(0))
}
