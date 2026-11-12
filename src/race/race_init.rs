//! `race::load_stage`'s "race init" (`drive::race_init`): after the best line,
//! the camera's start (`race::start_cars`), the face lights, the cars (`race::set_up_cars`), each
//! car's race state, the race slots, the cars' starts and randoms, then fifty settling steps
//! (`drive::race_cars_step`, `drive::share_all_cars`, the marks) and `race::grade_all_cars`. Here
//! its small steps.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::Trap;
use crate::session::PlainDword;

/// The car blocks `g_carBlocks` lists (all eight, whatever the race's count).
pub const BLOCKS: u32 = 8;
/// Car block n (`g_carBlocks`[n]).
pub fn block(shared: &RaceShared, n: u32) -> Result<&'static Car, BadInput> {
    shared.race_car(n as usize)
}

/// The car block's dword at `off`, for the offsets the race state and the overlay's clearing
/// name (`rc2k_core::race::cars::RACE_STATE`, `rc2k_core::race::panel::OVERLAY_CLEARED`).
pub fn state_cell(car: &Car, off: u32) -> Result<&PlainDword, BadInput> {
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
pub fn clear_race_slots(shared: &RaceShared) {
    for slot in &shared.work.race_slots {
        slot.part.set(0);
    }
}

/// `race::reset_car_starts` (0x444BBC): each car block's +0x14 and +8 0 (0x320 while `in_credits`
/// is set), +0x18 and +0x1C 0.
pub fn reset_car_starts(shared: &RaceShared) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::start_value`.
    let value = rc2k_core::race::cars::start_value(shared.work.in_credits.is_set());
    for n in 0..BLOCKS {
        {
            block(shared, n)?.run_ticks.set(value);
            state_cell(block(shared, n)?, 8)?.set(value);
            block(shared, n)?.start_in_gear.set(0);
            block(shared, n)?.false_start_count.set(0);
        }
    }
    Ok(())
}

/// `race::seed_car_randoms` (0x444BF6): each car block's +0x10 and +0xC one `nonzero_random`.
pub fn seed_car_randoms(shared: &RaceShared) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::car_seeds`.
    use crate::math::next_random::{DRAWS, wait_for_random, with_random};
    wait_for_random();
    DRAWS.fetch_add(BLOCKS, std::sync::atomic::Ordering::SeqCst);
    let seeds = with_random(rc2k_core::race::cars::car_seeds);
    for (n, &seed) in seeds.iter().enumerate() {
        {
            block(shared, n as u32)?.car_flags.set(seed);
            block(shared, n as u32)?.noise_word.set(seed);
        }
    }
    Ok(())
}

/// `race::reset_car_race_state` (0x444C1D): each car block's +0x71C 0xCA, +0x72C 0x12C,
/// +0x720/+0x724/+0x728/+0x730 0, +0xA0C .. +0xA1C -1; the four text dwords at 0x6D1838 .. spaces,
/// the four at 0x6D1850 .. -1.
pub fn reset_car_race_state(race: &Race) -> Result<(), BadInput> {
    let (shared, frame) = (&race.shared, &race.frame);
    // Converted: `rc2k_core::race::cars::{RACE_STATE, BLANK_TEXT, UNSET_TEXT}`.
    use rc2k_core::race::cars::{BLANK_TEXT, RACE_STATE, UNSET_TEXT};
    for n in 0..BLOCKS {
        for (off, v) in RACE_STATE {
            state_cell(block(shared, n)?, off)?.set(v);
        }
    }
    // `label_behind`, a64, `label_ahead`, a32: four dwords of spaces.
    for cell in [
        &frame.panel_and_overlay.get().overlay.label_behind[0],
        &frame.panel_and_overlay.get().overlay.label_behind[1],
        &frame.panel_and_overlay.get().overlay.label_ahead[0],
        &frame.panel_and_overlay.get().overlay.label_ahead[1],
    ] {
        cell.set(BLANK_TEXT);
    }
    // `kept_behind`, a64_0, `kept_ahead`, a32_0: four dwords of -1.
    for cell in [
        &frame.panel_and_overlay.get().overlay.kept_behind[0],
        &frame.panel_and_overlay.get().overlay.kept_behind[1],
        &frame.panel_and_overlay.get().overlay.kept_ahead[0],
        &frame.panel_and_overlay.get().overlay.kept_ahead[1],
    ] {
        cell.set(UNSET_TEXT);
    }
    Ok(())
}

/// `race::first_check_checkpoint` (0x446280): the first checkpoint from 1 whose face's +0x26 is not
/// 0, looked for up to `g_lastCheckpoint` (not included); 0 for none or with `g_checkpointCount` -
/// 2 negative. Refuses a look that would run past the list (the original runs on until it finds
/// one: `g_lastCheckpoint` 0 or 1).
pub fn first_check_checkpoint(course: &Course, shared: &RaceShared) -> Result<u32, Refused> {
    // Converted: `rc2k_core::race::checkpoints::first_check_checkpoint`.
    let count: u32 = shared.vars.checkpoint_count.get();
    let last = || shared.vars.last_checkpoint.get();
    let trap = Trap::new();
    let look = rc2k_core::race::checkpoints::first_check_checkpoint(count, last, |n| {
        let face: u32 =
            crate::race::checkpoints_store::face(&shared.buffers.checkpoints, n as usize).get();
        trap.keep_read(crate::race::course_places::face_number(course, face))
            .check()
    })?;
    trap.finish()?;
    Ok(look.and_then(|l| l.found).unwrap_or(0))
}
