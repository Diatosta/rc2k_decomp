//! `physics::compare_split_times` (0x436E11, race mode 3): the car's time (+0x88) against
//! the ten kept for this split (`unk_604134` + 0x28 x the split +0xAC): the place
//! +0xA0C counts down from 11 for each faster one (11 or more becomes -1); the nearest
//! gap behind (x10, the largest below 0) into +0xA14 with its name `dword_60410C[c]` in
//! +0xA1C; the nearest ahead (x10, the smallest) negated into +0xA10, its name in +0xA18;
//! `dword_6043F8` the time; for car 0 or 1 the time kept in `unk_75B02C` / `unk_75B09C`
//! [split].
//!
//! In:  nothing
//! Out: eax, ebx (split x 0x28), ecx, esi, edi as left; no flags read (the caller
//!      returns)
//!
//! Caller: `physics::split_by_mode`. Refuses (before anything) a split past 27 (the kept
//! tables' size not worked out).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use rc2k_core::physics::splits::{LAST_COMPARED_SPLIT, compare_split};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::compare_split`.
pub fn compare_split_times(race: &Race) -> Result<(), Refused> {
    if race.shared.car.splits_recorded.get() > LAST_COMPARED_SPLIT {
        return Err(BadInput("a split past 27").into());
    }
    let time = race.shared.car.stage_time.get();
    race.course.league.car_time.set(time);
    // Splits 16 .. 27 read past the sixteen rows (a slip of the original, kept).
    let first = race.shared.car.splits_recorded.get().wrapping_mul(10) as usize;
    let mut kept = [0u32; 10];
    for (k, part) in kept.iter_mut().enumerate() {
        *part = crate::session::read_on_in(
            &race.course.league,
            &race.course.league.rows,
            first + k,
            "a kept split time",
        )?;
    }
    let c = compare_split(time, &kept, |c| race.course.league.names[c].get());
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
        race.shared.car.split_gaps[k].set(v);
    }
    let car = race.shared.work.current_car.get();
    let times = match car {
        0 => &race.shared.state.results.results[0],
        1 => &race.shared.state.results.results[1],
        _ => return Ok(()),
    };
    // From the record's data on, and no further than the record.
    let at = std::mem::offset_of!(crate::race::state_store::RaceResult, data)
        + 4 * race.shared.car.splits_recorded.get() as usize;
    crate::pointers::GamePtr::to(times)
        .offset(at as u32)
        .write(time, "a result's split times")?;
    Ok(())
}
