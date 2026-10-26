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
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::menu::player::{PLAYERS_RVA, player_at};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};

const PAIRS_RVA: usize = rva(0x7587C8); // dword_7587C8: four (offset, value) pairs
const OUTER_RVA: usize = rva(0x7559B4); // dword_7559B4
const INNER_RVA: usize = rva(0x7559B8); // dword_7559B8
const ORDER_RVA: usize = rva(0x758720); // g_duelFirst, g_duelSecond, dword_758728, dword_75872C
const PLACE_POINTERS_RVA: usize = rva(0x758982); // off_758982, by byte offset
const POSITIONS_RVA: usize = rva(0x758768); // dword_758768 ..
const TOTALS_RVA: usize = rva(0x7587B8); // dword_7587B8 ..
const BEFORE_RVA: usize = rva(0x7587A8); // dword_7587A8 ..
const POINTS_RVA: usize = rva(0x758778); // dword_758778 ..
const BONUS_RVA: usize = rva(0x758758); // dword_758758 ..
const EXTRA_RVA: usize = rva(0x758788); // dword_758788 ..
const SUM_RVA: usize = rva(0x758798); // dword_758798 ..
const POINTS_TABLE_RVA: usize = rva(0x75892A); // dword_75892A
const POSITION_LIMIT: u32 = 16;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}

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
    match g(PLACE_POINTERS_RVA + offset as usize) {
        0 => Err(BadInput("a null place pointer").into()),
        to => Ok(to),
    }
}

/// The ranking of `rank_stage_times` (`by_totals` false) or `rank_totals`.
pub fn rank(by_totals: bool) -> Result<(), Refused> {
    let values: Vec<u32> = if by_totals {
        (0..4).map(|k| g(TOTALS_RVA + 4 * k)).collect()
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
        p(PAIRS_RVA + 8 * k, o);
        p(PAIRS_RVA + 8 * k + 4, v);
        p(ORDER_RVA + 4 * k, o);
    }
    p(OUTER_RVA, 3);
    p(INNER_RVA, 4);
    for (k, &to) in targets.iter().enumerate() {
        unsafe { (to as *mut u32).write_unaligned(k as u32 + 1) };
    }
    Ok(())
}

/// The stage record's +0x34, the time the bonus is counted from.
fn stage_time() -> Result<u32, Refused> {
    let table = g(rva(0x735512) + 4 * g(rva(0x7558C8)) as usize);
    if table == 0 {
        return Err(BadInput("no stage records").into());
    }
    let records = get(table.wrapping_add(4 * g(rva(0x7558D0))));
    if records == 0 {
        return Err(BadInput("no stage records").into());
    }
    Ok(get(records
        .wrapping_add(g(rva(0x7558CC)).wrapping_mul(0xA4))
        + 0x34))
}

fn points(position: u32) -> Result<u32, Refused> {
    if position >= POSITION_LIMIT {
        return Err(BadInput("a position past the points table").into());
    }
    Ok(g(POINTS_TABLE_RVA + 4 * position as usize))
}

/// `menu::score_four_players` (`n` 4) or `score_two_players` (`n` 2).
pub fn score(n: usize) -> Result<(), Refused> {
    let records = players(n as u32)?;
    let target = stage_time()?;
    let times: Vec<u32> = records.iter().map(|&r| get(r + 0x88)).collect();
    let positions: Vec<u32> = if n == 2 {
        rc2k_core::menu::results::two_places([times[0], times[1]]).to_vec()
    } else {
        (0..n).map(|k| g(POSITIONS_RVA + 4 * k)).collect()
    };
    let earned: Vec<u32> = positions
        .iter()
        .map(|&q| points(q))
        .collect::<Result<_, _>>()?;
    for k in 0..n {
        p(BEFORE_RVA + 4 * k, g(TOTALS_RVA + 4 * k));
    }
    if n == 2 {
        p(INNER_RVA, times[0]);
        p(POSITIONS_RVA, positions[0]);
        p(POSITIONS_RVA + 4, positions[1]);
    }
    // Converted: `rc2k_core::menu::results::score`.
    for k in 0..n {
        let sc = rc2k_core::menu::results::score(
            g(BEFORE_RVA + 4 * k),
            earned[k],
            times[k],
            target,
            get(records[k] + 0x94),
        );
        p(POINTS_RVA + 4 * k, sc.points);
        p(BONUS_RVA + 4 * k, sc.bonus);
        p(EXTRA_RVA + 4 * k, sc.extra);
        p(SUM_RVA + 4 * k, sc.sum);
        p(TOTALS_RVA + 4 * k, sc.total);
    }
    Ok(())
}

/// Everything they write.
pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(PAIRS_RVA) as u32, 0x20),
        at(OUTER_RVA),
        at(INNER_RVA),
        (game_addr(ORDER_RVA) as u32, 0x10),
        (game_addr(BONUS_RVA) as u32, 0x50), // 758758 .. 7587A8: bonus, positions, points, extra, sum
        (game_addr(BEFORE_RVA) as u32, 0x20), // 7587A8 .. 7587C8: before, totals
    ]
}

pub fn dispatch_results(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
) {
    dispatch_state(hook, r, call_original, &regions(), rust);
}

/// The self-test of each: player records of the test's in `g_players`' four places, the
/// times and totals of several patterns (ties, signs, below and above the stage's time),
/// the game's stage record (skipped without one), put back.
pub fn test_results(
    hook: &RawHook,
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let target = match stage_time() {
        Ok(v) => v,
        Err(_) => return t.skip("no stage records"),
    };
    let inputs = [(game_addr(PLAYERS_RVA) as u32, 0x10)];
    let written = regions();
    let saved = (capture(&inputs), capture(&written));
    let mut records = vec![[0u32; 0x40]; 4];
    let at_record: Vec<u32> = records.iter_mut().map(|r| r.as_mut_ptr() as u32).collect();
    let patterns: [[i32; 4]; 6] = [
        [100, 200, 300, 400],
        [400, 300, 200, 100],
        [250, 250, 100, 250],
        [-5, 0x7FFF_FFFF, i32::MIN, 3],
        [0, 0, 0, 0],
        [1000, 999, 1001, 1000],
    ];
    for (n, times) in patterns.iter().enumerate() {
        for shift in [0i32, 1] {
            check_state(
                &mut t,
                &format!("pattern {n}, shifted {shift}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    for (k, &r) in at_record.iter().enumerate() {
                        let time = (target as i32)
                            .wrapping_add(times[k])
                            .wrapping_sub(500 * shift);
                        ((r + 0x88) as *mut u32).write_unaligned(time as u32);
                        ((r + 0x94) as *mut u32).write_unaligned(0x100 * k as u32 + n as u32);
                        game_write(PLAYERS_RVA + 4 * k, r);
                        p(TOTALS_RVA + 4 * k, times[(k + 1) % 4] as u32);
                        p(POSITIONS_RVA + 4 * k, ((k + n) % 5) as u32);
                    }
                },
                &written,
                rust,
            );
        }
    }
    restore(&inputs, &saved.0);
    restore(&written, &saved.1);
    drop(records);
    t.finish()
}

/// The Rust versions as the harness runs them.
pub fn rank_stage_times(_: &Regs) -> Result<Left, Refused> {
    rank(false)?;
    Ok(Left::nothing())
}

pub fn rank_totals(_: &Regs) -> Result<Left, Refused> {
    rank(true)?;
    Ok(Left::nothing())
}

pub fn score_four_players(_: &Regs) -> Result<Left, Refused> {
    score(4)?;
    Ok(Left::nothing())
}

pub fn score_two_players(_: &Regs) -> Result<Left, Refused> {
    score(2)?;
    Ok(Left::nothing())
}
