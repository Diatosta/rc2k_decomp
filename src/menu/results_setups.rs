//! The set-ups of the players' results screens:
//! - `rank_stage_times` / `rank_totals`: the four players sorted by stage time (or total); the
//!   order goes into `results.order` and the places 1 .. 4 into the players' figures.
//! - `score_four_players`: per player the total before, the place's points, the time bonus (the
//!   stage record's time less the player's, 0 when greater), the player's top speed as extra
//!   points, their sum and the new total.
//! - `score_two_players`: the same for two, with the places worked out here (1 and 2, or 2 and 1
//!   when the second's time is below the first's).
//!
//! Refused before anything is written: a missing player record or stage record, a position past
//! 16, an offset past the four.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CaptionTables;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::results_store::PlayerFigures;
use crate::system::owner::System;

const POSITION_LIMIT: u32 = 16;

fn players(menus: &Menus, n: u32) -> Result<Vec<crate::menu::standings::PlayerCell<'_>>, Refused> {
    (0..n)
        .map(|k| {
            crate::menu::player::player(menus, k)
                .ok_or_else(|| BadInput("a player past the four").into())
        })
        .collect()
}

/// The figures of the player whose record is at byte `offset` of the player table.
fn figures_at(menus: &Menus, offset: u32) -> Result<&PlayerFigures, Refused> {
    if offset > 12 || !offset.is_multiple_of(4) {
        return Err(BadInput("an offset past the four players").into());
    }
    Ok(menus.play.results.player(offset as usize / 4)?)
}

/// The ranking of `rank_stage_times` (`by_totals` false) or `rank_totals`.
pub fn rank(menus: &Menus, by_totals: bool) -> Result<(), Refused> {
    let values: Vec<u32> = if by_totals {
        menus
            .play
            .results
            .players
            .iter()
            .map(|p| p.total.get())
            .collect()
    } else {
        players(menus, 4)?.iter().map(|&r| r.get().time).collect()
    };
    let pairs =
        rc2k_core::menu::results::rank([values[0], values[1], values[2], values[3]], by_totals);
    let targets: Vec<_> = pairs
        .iter()
        .map(|&(o, _)| figures_at(menus, o))
        .collect::<Result<_, _>>()?;
    for (k, &(o, _)) in pairs.iter().enumerate() {
        menus.play.results.order[k].set(o);
    }
    menus.screens.draw_work.button_y.set(3);
    for (k, to) in targets.iter().enumerate() {
        to.position.set(k as u32 + 1);
    }
    Ok(())
}

/// The stage record's +0x34, the time the bonus is counted from.
fn stage_time(screens: &Screens, system: &System) -> Result<u32, Refused> {
    let records = system.records().stage_tables.group_event_table(
        screens.event.stage.group.get(),
        screens.event.stage.sub.get(),
    )?;
    Ok(crate::records::stages_store::row(
        &system.records().stage_tables,
        records,
        screens.event.stage.number.get(),
    )?
    .record(&system.records().stage_tables)?
    .race_stage())
}

fn points(captions: &CaptionTables, position: u32) -> Result<u32, Refused> {
    if position >= POSITION_LIMIT {
        return Err(BadInput("a position past the points table").into());
    }
    Ok(captions.place_points[position as usize].get())
}

/// `menu::score_four_players` (`n` 4) or `score_two_players` (`n` 2).
pub fn score(menus: &Menus, system: &System, n: usize) -> Result<(), Refused> {
    let records = players(menus, n as u32)?;
    let target = stage_time(&menus.screens, system)?;
    let times: Vec<u32> = records.iter().map(|&r| r.get().time).collect();
    let positions: Vec<u32> = if n == 2 {
        rc2k_core::menu::results::two_places([times[0], times[1]]).to_vec()
    } else {
        (0..n)
            .map(|k| menus.play.results.players[k].position.get())
            .collect()
    };
    let earned: Vec<u32> = positions
        .iter()
        .map(|&q| points(&menus.tables.captions, q))
        .collect::<Result<_, _>>()?;
    let results = &menus.play.results;
    for player in &results.players[..n] {
        player.before.set(player.total.get());
    }
    if n == 2 {
        results.players[0].position.set(positions[0]);
        results.players[1].position.set(positions[1]);
    }
    for (k, player) in results.players[..n].iter().enumerate() {
        let sc = rc2k_core::menu::results::score(
            player.before.get(),
            earned[k],
            times[k],
            target,
            records[k].get().figure,
        );
        player.points.set(sc.points);
        player.time_bonus.set(sc.bonus);
        player.extra.set(sc.extra);
        player.sum.set(sc.sum);
        player.total.set(sc.total);
    }
    Ok(())
}

pub fn rank_stage_times(menus: &Menus) -> Result<(), Refused> {
    rank(menus, false)
}

pub fn rank_totals(menus: &Menus) -> Result<(), Refused> {
    rank(menus, true)
}

pub fn score_four_players(menus: &Menus, system: &System) -> Result<(), Refused> {
    score(menus, system, 4)
}

pub fn score_two_players(menus: &Menus, system: &System) -> Result<(), Refused> {
    score(menus, system, 2)
}
