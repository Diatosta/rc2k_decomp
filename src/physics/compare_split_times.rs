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
use crate::physics::car_store::CAR;
use crate::physics::cells::get_list;
use crate::race::league_store::LEAGUE;
use crate::race::state_store::RACE_STATE;
use crate::race::work_store::RACE_WORK;
use rc2k_core::physics::splits::{LAST_COMPARED_SPLIT, compare_split};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::compare_split`.
pub fn compare_split_times() -> Result<(), Refused> {
    if CAR.splits_recorded.get() > LAST_COMPARED_SPLIT {
        return Err(BadInput("a split past 27").into());
    }
    let time = CAR.stage_time.get();
    LEAGUE.car_time.set(time);
    // Splits 16 .. 27 read past the sixteen rows (a slip of the original, kept).
    let list = (LEAGUE.rows[0].addr()).wrapping_add(CAR.splits_recorded.get().wrapping_mul(0x28));
    let kept: [u32; 10] = { get_list(list) };
    let c = compare_split(time, &kept, |c| LEAGUE.names[c].get());
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
        CAR.split_gaps[k].set(v);
    }
    let car = RACE_WORK.current_car.get();
    let times = match car {
        0 => RACE_STATE.results.results[0].data.addr(),
        1 => RACE_STATE.results.results[1].data.addr(),
        _ => return Ok(()),
    };
    crate::pointers::GamePtr::new(times + 4 * CAR.splits_recorded.get())
        .write(time, "a result's split times")?;
    Ok(())
}
