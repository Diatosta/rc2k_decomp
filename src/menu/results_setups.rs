//! The set-ups of the players' results screens (agent E, 2026-10-19), all `pusha` ..
//! `popa` (every register kept), all plain stores:
//! - `menu::rank_stage_times` (0x48340B): the four players' stage times (`g_players`[n] +
//!   0x88) with their byte offsets (0, 4, 8, 12) in the pairs at `dword_7587C8`, sorted by
//!   the original's exchange loop (each pair i < j exchanged when j's time is at most i's,
//!   signed; `dword_7559B4` / `dword_7559B8` its counters, left 3 and 4); the offsets in
//!   that order into `g_duelFirst`, `g_duelSecond`, `dword_758728`, `dword_75872C`; and
//!   through the pointers `off_758982`[offset] (the four positions `dword_758768` ..) the
//!   places 1 .. 4.
//! - `menu::rank_totals` (0x48356F): the same on the totals `dword_7587B8` ..
//!   `dword_7587C4`, exchanged when j's is at least i's.
//! - `menu::score_four_players` (0x485CBB): for each of the four: the total before
//!   (`dword_7587A8`.. = `dword_7587B8`..), the place's points (`dword_75892A`[position]
//!   into `dword_758778`..), the time bonus (`dword_758758`..: the stage record's +0x34
//!   less the player's time, 0 when the time is greater, signed; the record
//!   `off_735512[dword_7558C8][dword_7558D0]` + `dword_7558CC` x 0xA4), the player's +0x94
//!   (`dword_758788`..), their sum (`dword_758798`..) and the new total (`dword_7587B8`..).
//! - `menu::score_two_players` (0x485B0E): the same for two, with the places worked out
//!   here: 1 and 2, or 2 and 1 when the second's time is below the first's (signed;
//!   `dword_7559B8` keeps the first's time).
//!
//! Refusals (before anything): a missing player record, a stage record missing on the
//! way, a position past 16 (the points table is read unchecked), an offset past the four
//! (the pointer table likewise), a null place pointer.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::player::player_at;
use crate::menu::results_store::RESULT_FIGURES;

const POSITION_LIMIT: u32 = 16;

fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

fn players(n: u32) -> Result<Vec<u32>, Refused> {
    (0..n)
        .map(|k| match player_at(k) {
            0 => Err(BadInput("a missing player record").into()),
            r => Ok(r),
        })
        .collect()
}

fn place_pointer(offset: u32) -> Result<u32, Refused> {
    if offset > 12 || !offset.is_multiple_of(4) {
        return Err(BadInput("an offset past the place pointers").into());
    }
    match CAPTIONS.place_pointers[offset as usize / 4].get() {
        0 => Err(BadInput("a null place pointer").into()),
        to => Ok(to),
    }
}

/// The ranking of `rank_stage_times` (`by_totals` false) or `rank_totals`.
pub fn rank(by_totals: bool) -> Result<(), Refused> {
    let values: Vec<u32> = if by_totals {
        (0..4).map(|k| RESULT_FIGURES.totals[k].get()).collect()
    } else {
        players(4)?.iter().map(|&r| get(r + 0x88)).collect()
    };
    // Converted: `rc2k_core::menu::results::rank`.
    let pairs =
        rc2k_core::menu::results::rank([values[0], values[1], values[2], values[3]], by_totals);
    let targets: Vec<u32> = pairs
        .iter()
        .map(|&(o, _)| place_pointer(o))
        .collect::<Result<_, _>>()?;
    for (k, &(o, v)) in pairs.iter().enumerate() {
        RESULT_FIGURES.pairs[2 * k].set(o);
        RESULT_FIGURES.pairs[2 * k + 1].set(v);
        RESULT_FIGURES.order[k].set(o);
    }
    DRAW_WORK.work_b4.set(3);
    DRAW_WORK.work_b8.set(4);
    for (k, &to) in targets.iter().enumerate() {
        unsafe { (to as *mut u32).write_unaligned(k as u32 + 1) };
    }
    Ok(())
}

/// The stage record's +0x34, the time the bonus is counted from.
fn stage_time() -> Result<u32, Refused> {
    let table =
        crate::records::stages_store::STAGE_TABLES.group_list(EVENT_STATE.stage_group.get());
    if table == 0 {
        return Err(BadInput("no stage records").into());
    }
    let records = get(table.wrapping_add(4 * EVENT_STATE.stage_sub.get()));
    if records == 0 {
        return Err(BadInput("no stage records").into());
    }
    Ok(get(records.wrapping_add(
        EVENT_STATE.stage_number.get().wrapping_mul(0xA4),
    ) + 0x34))
}

fn points(position: u32) -> Result<u32, Refused> {
    if position >= POSITION_LIMIT {
        return Err(BadInput("a position past the points table").into());
    }
    Ok(CAPTIONS.place_points[position as usize].get())
}

/// `menu::score_four_players` (`n` 4) or `score_two_players` (`n` 2).
pub fn score(n: usize) -> Result<(), Refused> {
    let records = players(n as u32)?;
    let target = stage_time()?;
    let times: Vec<u32> = records.iter().map(|&r| get(r + 0x88)).collect();
    let positions: Vec<u32> = if n == 2 {
        rc2k_core::menu::results::two_places([times[0], times[1]]).to_vec()
    } else {
        (0..n).map(|k| RESULT_FIGURES.positions[k].get()).collect()
    };
    let earned: Vec<u32> = positions
        .iter()
        .map(|&q| points(q))
        .collect::<Result<_, _>>()?;
    for k in 0..n {
        RESULT_FIGURES.before[k].set(RESULT_FIGURES.totals[k].get());
    }
    if n == 2 {
        DRAW_WORK.work_b8.set(times[0]);
        RESULT_FIGURES.positions[0].set(positions[0]);
        RESULT_FIGURES.positions[1].set(positions[1]);
    }
    // Converted: `rc2k_core::menu::results::score`.
    for k in 0..n {
        let sc = rc2k_core::menu::results::score(
            RESULT_FIGURES.before[k].get(),
            earned[k],
            times[k],
            target,
            get(records[k] + 0x94),
        );
        RESULT_FIGURES.points[k].set(sc.points);
        RESULT_FIGURES.time_bonus[k].set(sc.bonus);
        RESULT_FIGURES.extra[k].set(sc.extra);
        RESULT_FIGURES.sums[k].set(sc.sum);
        RESULT_FIGURES.totals[k].set(sc.total);
    }
    Ok(())
}

/// The Rust versions as the harness runs them.
pub fn rank_stage_times() -> Result<(), Refused> {
    rank(false)
}

pub fn rank_totals() -> Result<(), Refused> {
    rank(true)
}

pub fn score_four_players() -> Result<(), Refused> {
    score(4)
}

pub fn score_two_players() -> Result<(), Refused> {
    score(2)
}
