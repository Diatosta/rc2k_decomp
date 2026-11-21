//! Race mode 3: the car's time against the ten kept for this split (a replay's own table when it
//! has one). The place counts down from 11
//! for each faster time (11 or more becomes -1); the nearest gap behind (times 10, the largest
//! below 0) goes to one field with its name in another; the nearest ahead (times 10, the smallest)
//! negated goes to the next pair; `car_time` is the time; for car 0 or 1 the time is kept in the
//! results' data for the split.
//!
//! Refuses (before anything) a split past 27 (each kept table is 28 dwords).
use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::league_store::League;
use crate::race::state_store::RaceState;
use crate::race::work_store::RaceWork;
use rc2k_core::drive::replay_frames::plays_inputs;
use rc2k_core::physics::splits::{LAST_COMPARED_SPLIT, compare_split};

// Core: `rc2k_core::physics::splits::compare_split`.
pub fn compare_split_times(
    o: crate::owners::Owners<'static>,
    league: &mut League,
    work: &RaceWork,
    state: &RaceState,
    car: &Car,
) -> Result<(), Refused> {
    if car.splits_recorded.get() > LAST_COMPARED_SPLIT {
        return Err(BadInput("a split past 27").into());
    }
    // A replay compares against the table its race had; one written before it was kept, against
    // the table the game holds.
    if plays_inputs(o.race.course.replay_mode())
        && let Some(kept) = o
            .race
            .replay
            .v2()
            .lock()
            .ext
            .as_ref()
            .and_then(|e| e.league.clone())
    {
        league.names = kept.names;
        league.rows = kept.rows;
    }
    let time = car.stage_time.get();
    league.car_time = time;
    // Splits 16 .. 27 read past the sixteen rows (a slip kept as is).
    let first = car.splits_recorded.get().wrapping_mul(10) as usize;
    let mut kept = [0u32; 10];
    for (k, part) in kept.iter_mut().enumerate() {
        *part = league.kept_time(first + k)?;
    }
    let c = compare_split(time, &kept, |c| league.names[c]);
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
    // The players' cars (the first four); the computer's have no record.
    let times = match current {
        0..=3 => state.result(current as usize),
        _ => return Ok(()),
    };
    // From the record's data on, and no further than the record.
    times.set_split_time(car.splits_recorded.get() as usize, time)?;
    Ok(())
}
