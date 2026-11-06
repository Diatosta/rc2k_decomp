//! `race::load_stage`'s "race init" (`sub_4462B9`, IDA 2026-10-17): after the best line,
//! the camera's start (`sub_446463`), the face lights, the cars (`sub_43603C`), each car's
//! race state, the race slots, the cars' starts and randoms, then fifty settling steps
//! (`sub_4608BB`, `sub_456484`, the marks) and `sub_43FDFE`. Here its small steps.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::race;
use crate::race::trap::Trap;
use crate::race::trap::car_at;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::session::Dword;

/// The car blocks `g_carBlocks` lists (all eight, whatever the race's count).
pub const BLOCKS: u32 = 8;
/// Car block n (`g_carBlocks`[n]).
pub fn block(n: u32) -> u32 {
    RACE_WORK.car_block(n as usize)
}

/// The car block's dword at `off`, for the offsets the race state and the overlay's clearing
/// name (`rc2k_core::race::cars::RACE_STATE`, `rc2k_core::race::panel::OVERLAY_CLEARED`).
pub fn state_cell(car: &Car, off: u32) -> Result<&Dword, BadInput> {
    Ok(match off {
        0x8 => &car.step_clock,
        0x71C => &car.notice_id,
        0x720 => &car.notice_counter,
        0x724 => &car.notice_last,
        0x728 => &car.notice_shown,
        0x72C => &car.notice_time,
        0x730 => &car.notice_time_b,
        0xA0C..=0xA1C if off.is_multiple_of(4) => &car.split_gaps[((off - 0xA0C) / 4) as usize],
        0xF78..=0xF94 if off.is_multiple_of(4) => {
            &car.car_event_codes[((off - 0xF78) / 4) as usize]
        }
        0xF98 => &car.car_event_count,
        0xFA0 => &car.car_event_position,
        _ => return Err(BadInput("a car block offset the race state does not name")),
    })
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
            state_cell(car_at(b)?, 8)?.set(value);
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
            state_cell(car_at(b)?, off)?.set(v);
        }
    }
    // dword_6D1838, a64, dword_6D1844, a32: four dwords of spaces.
    for cell in [
        &race().frame.panel_and_overlay.overlay.label_behind[0],
        &race().frame.panel_and_overlay.overlay.label_behind[1],
        &race().frame.panel_and_overlay.overlay.label_ahead[0],
        &race().frame.panel_and_overlay.overlay.label_ahead[1],
    ] {
        cell.set(BLANK_TEXT);
    }
    // dword_6D1850, a64_0, dword_6D185C, a32_0: four dwords of -1.
    for cell in [
        &race().frame.panel_and_overlay.overlay.kept_behind[0],
        &race().frame.panel_and_overlay.overlay.kept_behind[1],
        &race().frame.panel_and_overlay.overlay.kept_ahead[0],
        &race().frame.panel_and_overlay.overlay.kept_ahead[1],
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
        trap.keep_read(crate::race::course_places::face_number(
            face,
            "a checkpoint face",
        ))
        .check()
    })?;
    trap.finish()?;
    Ok(look.and_then(|l| l.found).unwrap_or(0))
}
