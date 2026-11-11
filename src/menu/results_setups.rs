//! The set-ups of the players' results screens, all plain stores:
//! - `menu::rank_stage_times`: the four players' stage times (`g_players`[n] + 0x88) with
//!   their byte offsets (0, 4, 8, 12) in `results.pairs`, sorted by the original's exchange
//!   loop (each pair i < j exchanged when j's time is at most i's, signed; its two counters
//!   are left at 3 and 4); the offsets in that order go into the duel's first and second
//!   players and the two entries after them; and through the place pointers
//!   (`captions.place_pointers`[offset], the four `results.positions`) the places 1 .. 4.
//! - `menu::rank_totals`: the same on `results.totals`, exchanged when j's is at least i's.
//! - `menu::score_four_players`: for each of the four: the total before (`results.before`
//!   takes `results.totals`), the place's points (`captions.place_points`[position] into
//!   `results.points`), the time bonus (`results.time_bonus`: the stage record's +0x34 less
//!   the player's time, 0 when the time is greater, signed; the record is the stage's row of
//!   the event's stage table), the player's +0x94 (`results.extra`), their sum
//!   (`results.sums`) and the new total (`results.totals`).
//! - `menu::score_two_players`: the same for two, with the places worked out here: 1 and 2,
//!   or 2 and 1 when the second's time is below the first's (signed; the second counter
//!   keeps the first's time).
//!
//! Refusals (before anything): a missing player record, a stage record missing on the
//! way, a position past 16 (the points table is read unchecked), an offset past the four
//! (the pointer table likewise), a null place pointer.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CaptionTables;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::system::owner::system;

const POSITION_LIMIT: u32 = 16;

fn players(menus: &Menus, n: u32) -> Result<Vec<crate::menu::standings::PlayerCell<'_>>, Refused> {
    (0..n)
        .map(|k| {
            crate::menu::player::player(menus, k)
                .ok_or_else(|| BadInput("a player past the four").into())
        })
        .collect()
}

fn place_pointer(menus: &Menus, offset: u32) -> Result<&crate::session::PlainDword, Refused> {
    if offset > 12 || !offset.is_multiple_of(4) {
        return Err(BadInput("an offset past the place pointers").into());
    }
    Ok(CaptionTables::place_of(menus, offset as usize / 4)?)
}

/// The ranking of `rank_stage_times` (`by_totals` false) or `rank_totals`.
pub fn rank(menus: &Menus, by_totals: bool) -> Result<(), Refused> {
    let values: Vec<u32> = if by_totals {
        (0..4).map(|k| menus.play.results.totals[k].get()).collect()
    } else {
        players(menus, 4)?.iter().map(|&r| r.get().time).collect()
    };
    // The logic is in `rc2k_core::menu::results::rank`.
    let pairs =
        rc2k_core::menu::results::rank([values[0], values[1], values[2], values[3]], by_totals);
    let targets: Vec<_> = pairs
        .iter()
        .map(|&(o, _)| place_pointer(menus, o))
        .collect::<Result<_, _>>()?;
    for (k, &(o, v)) in pairs.iter().enumerate() {
        menus.play.results.pairs[2 * k].set(o);
        menus.play.results.pairs[2 * k + 1].set(v);
        menus.play.results.order[k].set(o);
    }
    menus.screens.draw_work.scratch_b4.set(3);
    menus.screens.draw_work.scratch_b8.set(4);
    for (k, to) in targets.iter().enumerate() {
        to.set(k as u32 + 1);
    }
    Ok(())
}

/// The stage record's +0x34, the time the bonus is counted from.
fn stage_time(screens: &Screens) -> Result<u32, Refused> {
    let records = system().records.stage_tables.group_event_table(
        screens.event.stage_group.get(),
        screens.event.stage_sub.get(),
    )?;
    Ok(
        crate::records::stages_store::row(records, screens.event.stage_number.get())?
            .record()?
            .race_stage(),
    )
}

fn points(tables: &Tables, position: u32) -> Result<u32, Refused> {
    if position >= POSITION_LIMIT {
        return Err(BadInput("a position past the points table").into());
    }
    Ok(tables.captions.place_points[position as usize].get())
}

/// `menu::score_four_players` (`n` 4) or `score_two_players` (`n` 2).
pub fn score(menus: &Menus, n: usize) -> Result<(), Refused> {
    let records = players(menus, n as u32)?;
    let target = stage_time(&menus.screens)?;
    let times: Vec<u32> = records.iter().map(|&r| r.get().time).collect();
    let positions: Vec<u32> = if n == 2 {
        rc2k_core::menu::results::two_places([times[0], times[1]]).to_vec()
    } else {
        (0..n)
            .map(|k| menus.play.results.positions[k].get())
            .collect()
    };
    let earned: Vec<u32> = positions
        .iter()
        .map(|&q| points(&menus.tables, q))
        .collect::<Result<_, _>>()?;
    for k in 0..n {
        menus.play.results.before[k].set(menus.play.results.totals[k].get());
    }
    if n == 2 {
        menus.screens.draw_work.scratch_b8.set(times[0]);
        menus.play.results.positions[0].set(positions[0]);
        menus.play.results.positions[1].set(positions[1]);
    }
    // The logic is in `rc2k_core::menu::results::score`.
    for k in 0..n {
        let sc = rc2k_core::menu::results::score(
            menus.play.results.before[k].get(),
            earned[k],
            times[k],
            target,
            records[k].get().figure,
        );
        menus.play.results.points[k].set(sc.points);
        menus.play.results.time_bonus[k].set(sc.bonus);
        menus.play.results.extra[k].set(sc.extra);
        menus.play.results.sums[k].set(sc.sum);
        menus.play.results.totals[k].set(sc.total);
    }
    Ok(())
}

pub fn rank_stage_times(menus: &Menus) -> Result<(), Refused> {
    rank(menus, false)
}

pub fn rank_totals(menus: &Menus) -> Result<(), Refused> {
    rank(menus, true)
}

pub fn score_four_players(menus: &Menus) -> Result<(), Refused> {
    score(menus, 4)
}

pub fn score_two_players(menus: &Menus) -> Result<(), Refused> {
    score(menus, 2)
}
