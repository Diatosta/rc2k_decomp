//! The stage load's "race init" (`drive::race_init`): after the best line, the camera's start
//! (`race::start_cars`), the face lights, the cars (`race::set_up_cars`), each car's race state,
//! the race slots, the cars' starts and randoms, then fifty settling steps
//! (`drive::race_cars_step`, `drive::share_all_cars`, the marks) and `race::grade_all_cars`. This
//! file holds its small steps.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::Trap;
use rc2k_core::race::cars::BlockWord;

/// The car blocks `car_blocks` lists (all eight, whatever the race's count).
pub const BLOCKS: u32 = 8;
/// Car block n (`car_blocks`[n]).
pub fn block(shared: &RaceShared, n: u32) -> Result<&Car, BadInput> {
    shared.race_car(n as usize)
}

/// The car block's dword the core's race state and overlay tables name.
pub(super) fn block_word(car: &Car, word: BlockWord) -> Result<&PlainDword, BadInput> {
    const OUT: BadInput = BadInput("a car block entry the race state does not have");
    Ok(match word {
        BlockWord::NoticeId => &car.notice_id,
        BlockWord::NoticeCounter => &car.notice_counter,
        BlockWord::NoticeLast => &car.notice_last,
        BlockWord::NoticeShown => &car.notice_shown,
        BlockWord::NoticeTime => &car.notice_time,
        BlockWord::NoticeTimeB => &car.notice_time_b,
        BlockWord::SplitGap(n) => car.split_gaps.get(n).ok_or(OUT)?,
        BlockWord::EventCode(n) => car.car_event_codes.get(n).ok_or(OUT)?,
        BlockWord::EventCount => &car.car_event_count,
        BlockWord::EventPosition => &car.car_event_position,
    })
}

/// Zeroes the first dword of the eight `race_slots` records.
pub fn clear_race_slots(shared: &RaceShared) {
    for slot in &mut shared.work_mut().race_slots {
        slot.part = 0;
    }
}

/// Resets each car block's starts: `run_ticks` and `step_clock` to 0 (0x320 while `in_credits` is
/// set), `start_in_gear` and `false_start_count` to 0.
pub fn reset_car_starts(shared: &RaceShared) -> Result<(), BadInput> {
    // Logic in `rc2k_core::race::cars::start_value`.
    let value = rc2k_core::race::cars::start_value(shared.work().in_credits != 0);
    for n in 0..BLOCKS {
        {
            block(shared, n)?.run_ticks.set(value);
            block(shared, n)?.step_clock.set(value);
            block(shared, n)?.start_in_gear.set(0);
            block(shared, n)?.false_start_count.set(0);
        }
    }
    Ok(())
}

/// Gives each car block's `car_flags` and `noise_word` one `nonzero_random`.
pub fn seed_car_randoms(shared: &RaceShared) -> Result<(), BadInput> {
    // Logic in `rc2k_core::race::cars::car_seeds`.
    let seeds = shared.random.with_random(rc2k_core::race::cars::car_seeds);
    for (n, &seed) in seeds.iter().enumerate() {
        {
            block(shared, n as u32)?.car_flags.set(seed);
            block(shared, n as u32)?.noise_word.set(seed);
        }
    }
    Ok(())
}

/// Resets each car block's race state: `notice_id` 0xCA, `notice_time` 0x12C,
/// `notice_counter` / `notice_last` / `notice_shown` and `notice_time_b` 0, `split_gaps` -1;
/// the four label dwords (`label_behind` and `label_ahead`) are set to spaces, the four kept copies
/// to -1.
pub fn reset_car_race_state(race: &Race) -> Result<(), BadInput> {
    let (shared, frame) = (&race.shared, &race.frame);
    // Logic in `rc2k_core::race::cars::{RACE_STATE, BLANK_TEXT, UNSET_TEXT}`.
    use rc2k_core::race::cars::{BLANK_TEXT, RACE_STATE, UNSET_TEXT};
    for n in 0..BLOCKS {
        for (word, v) in RACE_STATE {
            block_word(block(shared, n)?, word)?.set(v);
        }
    }
    // `label_behind` and `label_ahead`: four dwords of spaces.
    let mut overlay = frame.overlay_mut();
    overlay.label_behind = [BLANK_TEXT; 2];
    overlay.label_ahead = [BLANK_TEXT; 2];
    // `kept_behind` and `kept_ahead`: four dwords of -1.
    overlay.kept_behind = [UNSET_TEXT; 2];
    overlay.kept_ahead = [UNSET_TEXT; 2];
    Ok(())
}

/// Finds the first checkpoint from 1 whose face's check byte is not 0, looking up to
/// `last_checkpoint` (not included); 0 for none or with `checkpoint_count` - 2 negative. Refuses a
/// look that would run past the list.
pub fn first_check_checkpoint(course: &Course, shared: &RaceShared) -> Result<u32, Refused> {
    // Logic in `rc2k_core::race::checkpoints::first_check_checkpoint`.
    let count: u32 = shared.vars.get().checkpoint_count.get();
    let last = || shared.vars.get().last_checkpoint.get();
    let trap = Trap::new();
    let look = rc2k_core::race::checkpoints::first_check_checkpoint(count, last, |n| {
        let face: u32 = crate::race::checkpoints_store::face(shared.checkpoints(), n as usize);
        trap.keep_read(crate::race::course_places::face_number(
            course.reads(),
            face,
        ))
        .check()
    })?;
    trap.finish()?;
    Ok(look.and_then(|l| l.found).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::race::cars::RACE_STATE;
    use rc2k_core::race::panel::OVERLAY_CLEARED;

    /// Each name the core's tables use is a car block field of the same words: the notice
    /// fields, the split gaps and the panel's marks.
    #[test]
    fn block_words_are_the_old_fields() {
        let car = Box::new(Car::new());
        let state = [
            &car.notice_time_b,
            &car.notice_time,
            &car.notice_last,
            &car.notice_id,
            &car.notice_counter,
            &car.notice_shown,
            &car.split_gaps[0],
            &car.split_gaps[1],
            &car.split_gaps[2],
            &car.split_gaps[3],
            &car.split_gaps[4],
        ];
        for ((word, _), field) in RACE_STATE.into_iter().zip(state) {
            assert!(
                core::ptr::eq(block_word(&car, word).unwrap(), field),
                "{word:?}"
            );
        }
        let overlay = [
            &car.car_event_codes[0],
            &car.car_event_codes[1],
            &car.car_event_codes[2],
            &car.car_event_codes[3],
            &car.car_event_codes[4],
            &car.car_event_codes[5],
            &car.car_event_codes[6],
            &car.car_event_codes[7],
            &car.car_event_count,
            &car.car_event_position,
        ];
        for (word, field) in OVERLAY_CLEARED.into_iter().zip(overlay) {
            assert!(
                core::ptr::eq(block_word(&car, word).unwrap(), field),
                "{word:?}"
            );
        }
        assert!(block_word(&car, BlockWord::SplitGap(7)).is_err());
        assert!(block_word(&car, BlockWord::EventCode(8)).is_err());
    }
}
