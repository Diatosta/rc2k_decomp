//! `physics::compare_split_times` (0x436E11, race mode 3): the car's time (+0x88) against
//! the ten kept for this split (`rows` + 0x28 x the split +0xAC): the place
//! +0xA0C counts down from 11 for each faster one (11 or more becomes -1); the nearest
//! gap behind (x10, the largest below 0) into +0xA14 with its name `names`[c]` in
//! +0xA1C; the nearest ahead (x10, the smallest) negated into +0xA10, its name in +0xA18;
//! `car_time` the time; for car 0 or 1 the time kept in `unk_75B02C` / `unk_75B09C`
//! [split].
//!
//! Caller: `physics::split_by_mode`. Refuses (before anything) a split past 27 (each kept
//! table is 0x70 bytes: `unk_75B02C` .. `unk_75B09C`, 28 dwords, IDA).
use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::league_store::League;
use crate::race::state_store::RaceState;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::splits::{LAST_COMPARED_SPLIT, compare_split};

// Converted: `rc2k_core::physics::splits::compare_split`.
pub fn compare_split_times(
    league: &League,
    work: &RaceWork,
    state: &RaceState,
    car: &Car,
) -> Result<(), Refused> {
    if car.splits_recorded.get() > LAST_COMPARED_SPLIT {
        return Err(BadInput("a split past 27").into());
    }
    let time = car.stage_time.get();
    league.car_time.set(time);
    // Splits 16 .. 27 read past the sixteen rows (a slip of the original, kept).
    let first = car.splits_recorded.get().wrapping_mul(10) as usize;
    let mut kept = [0u32; 10];
    for (k, part) in kept.iter_mut().enumerate() {
        *part = crate::session::read_on_in(league, &league.rows, first + k, "a kept split time")?;
    }
    let c = compare_split(time, &kept, |c| league.names[c].get());
    let f = c.figures;
    for (k, v) in [
        f.place,
        f.gap_ahead,
        f.gap_behind,
        f.name_ahead,
        f.name_behind,
    ]
    .into_iter()
    .enumerate()
    {
        car.split_gaps[k].set(v);
    }
    let current = work.current_car.get();
    let times = match current {
        0 => &state.results.results[0],
        1 => &state.results.results[1],
        _ => return Ok(()),
    };
    // From the record's data on, and no further than the record.
    let at = std::mem::offset_of!(crate::race::state_store::RaceResult, data)
        + 4 * car.splits_recorded.get() as usize;
    crate::pointers::Store::store(times, at, &time.to_le_bytes())?;
    Ok(())
}
