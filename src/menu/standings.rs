//! The championship's standings tables as the event code walks them: table A
//! (`BestTables::ranked_times`, best times: `player` 1 .. 4 or 0 for a computer
//! driver, `group`, `figure` the position) and table B (`BestTables::champ_history`,
//! `rankings::HistoryEntry`); both end with an entry whose first dword is -1.
//! Also the drivers (-1 at the end) and the players.

use crate::errors::{BadInput, FatalError, Refused};
use crate::memory::session::PlainDword;
use crate::menu::drivers_store::Driver;
use crate::menu::owner::Play;
use crate::menu::owner::Screens;
use crate::menu::owner::{Cars, Menus};
pub use crate::system::owner::System;
use rc2k_core::menu::standings::{self, NAME_LEN};

use crate::saves::best_times::BestTables;
use rc2k_core::saves::rankings::{BKey, ChampHistory, HistoryEntry, RankedTimes};
/// "Any group" for the first-player searches.
pub const ANY_GROUP: u32 = standings::ANY_GROUP;

/// A player's record, typed.
pub type PlayerCell<'a> = &'a crate::memory::session::Shared<rc2k_core::menu::player::Player>;

/// Whether any of the players has no name (a null name place).
pub(super) fn any_unnamed(players: &[PlayerCell]) -> Result<bool, crate::errors::BadInput> {
    Ok(players.iter().any(|p| p.get().name_player().is_none()))
}

/// Player `n`'s record (`n` one of the four).
pub fn player(menus: &Menus, n: u32) -> PlayerCell<'_> {
    &menus.play.slots.records[n as usize]
}

/// `player` for a caller that holds only the players' slots.
pub(super) fn player_in(play: &Play, n: u32) -> PlayerCell<'_> {
    &play.slots.records[n as usize]
}

/// The drivers as a `do .. while` walks them: the first always, then each while its first
/// dword is not -1. The walk ends at the latest on the -1 after the 180 (`DriverTable.end`);
/// refused when that is gone.
pub fn drivers(menus: &Menus) -> Result<&[Driver], BadInput> {
    drivers_of(&menus.cars)
}

/// `drivers` for a caller that holds only the cars' state.
pub(super) fn drivers_of(cars: &Cars) -> Result<&[Driver], BadInput> {
    let all = &cars.drivers.drivers;
    let count = 1 + all[1..]
        .iter()
        .take_while(|d| d.name_entry.get() != u32::MAX)
        .count();
    if count == all.len() && cars.drivers.end.get() != u32::MAX {
        return Err(BadInput("the drivers without their end entry"));
    }
    Ok(&all[..count])
}

/// `f` on table A; refused without it.
pub fn with_a<R>(
    tables: &BestTables,
    f: impl FnOnce(&mut RankedTimes) -> R,
) -> Result<R, BadInput> {
    tables.ranked_times.with(f)
}

/// `f` on table B; refused without it.
pub(super) fn with_b<R>(
    tables: &BestTables,
    f: impl FnOnce(&mut ChampHistory) -> R,
) -> Result<R, BadInput> {
    tables.champ_history.with(f)
}

/// How many of table A's entries are walked; refused without the table or with no end entry
/// within the core's `MOST_WALKED`.
pub(super) fn walked_a(tables: &BestTables) -> Result<usize, BadInput> {
    with_a(tables, |t| t.walked())?.ok_or(BadInput("a standings table without its end entry"))
}

/// As `walked_a`, of table B.
pub(super) fn walked_b(tables: &BestTables) -> Result<usize, BadInput> {
    with_b(tables, |t| t.walked())?.ok_or(BadInput("a standings table without its end entry"))
}

/// The walked entries of table A, copied out.
pub(super) fn walked_entries_a(
    tables: &BestTables,
) -> Result<Vec<rc2k_core::saves::entries::BestTime>, BadInput> {
    let n = walked_a(tables)?;
    with_a(tables, |t| t.entries[..n].to_vec())
}

/// The walked entries of table B, copied out.
pub fn walked_entries_b(tables: &BestTables) -> Result<Vec<HistoryEntry>, BadInput> {
    let n = walked_b(tables)?;
    with_b(tables, |t| t.entries[..n].to_vec())
}

/// Which standings table a search walks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    A,
    B,
}

/// The search for the first player (`find1stplayer`) from the table's start: the entries of
/// `group` (`ANY_GROUP`: any entry whose group is not 0) whose player is not 0; counts the
/// group's computer drivers before it, its figure (the position) and the entry. None: the
/// fatal error `code` with `message`.
pub fn first_player(
    tables: &BestTables,
    which: Which,
    group: u32,
    code: u32,
    message: &'static str,
) -> Result<Found, Refused> {
    // The logic is in `rc2k_core::saves::rankings::first_player`, over the entries walked.
    use rc2k_core::saves::rankings::first_player as find;
    let found = match which {
        Which::A => find(&walked_entries_a(tables)?, group, |e| {
            (e.head.group, e.head.player, e.head.figure)
        }),
        Which::B => find(&walked_entries_b(tables)?, group, |e| {
            (e.group, e.player, e.position)
        }),
    };
    match found {
        Some(f) => Ok(Found {
            computers: f.computers,
            figure: f.figure,
            entry: f.entry,
        }),
        None => Err(FatalError { code, message }.into()),
    }
}

/// What the first-player searches find: `computers` (the group's computer drivers before it;
/// the player's number for `first_player_unplaced`), its figure, and the entry (here its
/// number in the table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub computers: u32,
    pub figure: u32,
    pub entry: usize,
}

/// The walked drivers, for the tables' fills.
pub(super) fn walked_drivers(
    cars: &Cars,
) -> Result<Vec<rc2k_core::saves::rankings::Driver>, BadInput> {
    Ok(drivers_of(cars)?
        .iter()
        .map(|d| rc2k_core::saves::rankings::Driver {
            number: d.name_entry.get(),
            group: d.class.get(),
            time: d.time.get(),
            total: d.total.get(),
            nation: d.nation.get(),
            car: d.car.get(),
            entered: d.in_event.get(),
            points: d.results.each_ref().map(PlainDword::get),
        })
        .collect())
}

/// A player as a stage record's holder: the `figure`, nationality, car and the name's
/// 0x20 bytes (copied past the NUL too).
pub(super) fn as_holder(
    play: &Play,
    p: &rc2k_core::menu::player::Player,
) -> Result<
    (
        crate::records::stages_store::HolderRow,
        [u8; crate::records::stages_store::HOLDER_NAME_LEN],
    ),
    BadInput,
> {
    let row = crate::records::stages_store::HolderRow {
        nation: p.nationality,
        car: p.car_number,
        speed: p.figure,
    };
    Ok((row, name_of(play, p)?))
}

/// The player's name, `N` bytes from its start (bounded by the names' store: a name that lacks
/// its end is read on past it).
pub fn name_of<const N: usize>(
    play: &Play,
    p: &rc2k_core::menu::player::Player,
) -> Result<[u8; N], BadInput> {
    let bytes = crate::menu::player_names::NameAt::of(name_player(p)?).tail(play);
    bytes
        .get(..N)
        .map(|b| std::array::from_fn(|i| b[i]))
        .ok_or(BadInput("a name past the names' area"))
}

/// Whose name the player has; refused without one.
pub fn name_player(p: &rc2k_core::menu::player::Player) -> Result<usize, BadInput> {
    crate::menu::player_names::name_of(p)
}

/// The player's name as the entries carry it.
pub(super) fn name_id_of(
    p: &rc2k_core::menu::player::Player,
) -> Result<rc2k_core::saves::name_id::NameId, BadInput> {
    Ok(rc2k_core::saves::name_id::NameId::of_player(name_player(
        p,
    )?))
}

/// Runs `f` on the kept player name, the sorts' swap buffer (a copy of it, put back after).
pub(super) fn with_kept_name<R>(play: &Play, f: impl FnOnce(&mut [u8; NAME_LEN]) -> R) -> R {
    let mut all = play.slots.names.kept_name.read();
    let [kept @ .., _] = &mut all;
    let r = f(kept);
    play.slots.names.kept_name.write(&all);
    r
}

/// The players `0 .. Event.player_count`: refused at 0, past 4 or with one missing.
pub(super) fn counted_players<'a>(
    screens: &Screens,
    play: &'a Play,
) -> Result<Vec<PlayerCell<'a>>, BadInput> {
    let n: u32 = screens.event.read().player_count;
    if n == 0 || n > 4 {
        return Err(BadInput("a player count of 0 or past 4"));
    }
    Ok((0..n).map(|k| player_in(play, k)).collect())
}

/// The current record's event (the record table, `section`, `index`; its `event_slot`): its place
/// in the stages' best times (`BestTables::stage_times`); refused past the table's
/// sections, with no table, or past it.
pub(super) fn current_event(
    screens: &Screens,
    system: &System,
    tables: &BestTables,
) -> Result<usize, BadInput> {
    let table = system.records().stage_tables.championship();
    let section: u32 = screens.event.read().section;
    let index: u32 = screens.event.read().stage_in_section;
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(
            &system.records().stage_tables,
            table,
            section,
            600,
        )?
    {
        return Err(BadInput("a record past the table"));
    }
    let record = crate::records::find_record::find_record_in(
        &system.records().stage_tables,
        table,
        index as i32,
        section,
    )?;
    let event = record.record(&system.records().stage_tables)?.event_slot() as usize;
    if !tables.stage_times.with(|t| event < t.stages.len())? {
        return Err(BadInput("an event past the stages' best times"));
    }
    Ok(event)
}

/// `f` on event `event`'s ten record entries (`current_event`'s place).
pub(super) fn with_event<R>(
    tables: &BestTables,
    event: usize,
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, BadInput> {
    tables
        .stage_times
        .with(|t| t.stages.get_mut(event).map(f))?
        .ok_or(BadInput("an event past the stages' best times"))
}

/// Group `group`'s tenth best score (`BestTables::scores`); refused with no table or
/// a group past the five.
pub(super) fn tenth_best_score(tables: &BestTables, group: u32) -> Result<u32, BadInput> {
    tables
        .scores
        .with(|t| t.groups.get(group as usize).map(|g| g[9].score))?
        .ok_or(BadInput("a group past the best scores' five"))
}
const SORTED: u32 = standings::SORTED as u32;

/// Standings table A's first 0xB4 entries sorted by `key` (signed; an exchange sort: an entry
/// later or equal moves up).
/// Refused without a table.
pub(super) fn sort_table_a(
    screens: &Screens,
    play: &Play,
    tables: &BestTables,
    key: rc2k_core::saves::entries::Field,
) -> Result<(), BadInput> {
    with_a(tables, |t| {
        with_kept_name(play, |kept| {
            rc2k_core::saves::rankings::sort_a(t, SORTED as usize, key, kept);
        });
    })?;

    screens.draw_work.write().button_y = SORTED - 1;

    Ok(())
}

/// The current player's (`Event.current_player`) best score entered at the bottom of `group`'s ten
/// (`BestTables::scores`, entry 9): its score (`score` of its record), its number
/// plus 1, `new` 1, the nationality and car, its class, its name pointer and the name's 0x1F
/// bytes; then moved up its group. Refused for a current player past 3 or missing, no name,
/// no table or a group past 4.
pub fn enter_best(
    screens: &Screens,
    play: &Play,
    tables: &BestTables,
    group: Option<u32>,
    score: fn(&rc2k_core::menu::player::Player) -> u32,
) -> Result<(), BadInput> {
    let n: u32 = screens.event.read().current_player;
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player_in(play, n);
    let group = group.unwrap_or(p.get().class);
    let scores = &tables.scores;
    let (true, true, false) = (
        scores.is_taken(),
        group < 5,
        p.get().name_player().is_none(),
    ) else {
        return Err(BadInput("no best-score table, a group past 4 or no name"));
    };
    use rc2k_core::saves::scores as s;
    let name_id = name_id_of(&p.get())?;
    let best = s::Best {
        player: n + 1,
        score: score(&p.get()),
        nation: p.get().nationality,
        car: p.get().car_number,
        group: p.get().class,
        name_id,
        name: name_of(play, &p.get())?,
    };
    scores.with(|t| {
        with_kept_name(play, |kept| {
            s::enter_best(&mut t.groups[group as usize], &best, kept);
        });
    })?;
    Ok(())
}

/// Standings table B's first 0xB4 entries sorted by `key`, as `sort_table_a`: going up (a
/// later entry not above an earlier one moves up) or, with `down`, going down (a later
/// entry not below one moves up). Refused without a table.
pub(super) fn sort_table_b(
    screens: &Screens,
    play: &Play,
    tables: &BestTables,
    key: BKey,
    down: bool,
) -> Result<(), BadInput> {
    with_b(tables, |t| {
        with_kept_name(play, |kept| {
            rc2k_core::saves::rankings::sort_b(t, SORTED as usize, key, down, kept);
        });
    })?;

    screens.draw_work.write().button_y = SORTED - 1;

    Ok(())
}

/// Whether the player's part `k` (an `INSPECTED` index) is at or past its limit (signed).
pub(super) fn past_limit(screens: &Screens, p: PlayerCell, k: u32) -> Result<bool, BadInput> {
    let limit = screens.state.read().driver_stats.limits[k as usize] as i32;
    Ok(p.get().damage[k as usize] as i32 >= limit)
}

/// What the numbering functions (`number_all_a`, `number_all_b`, `number_standings_a`,
/// `number_standings_b`) give: the last number given (the class passes store it).
#[derive(Clone, Copy)]
pub struct Numbered {
    pub last_number: u32,
}

/// The numbering of a standings table (`number_all_*`, `number_standings_*`): each entry
/// `which` counts numbered in turn, its position set and, for a player's, the player's
/// `number` field. Refused, before any write, when a counted entry's player is past 4 or
/// not there.
pub(super) fn number_table(
    play: &Play,
    tables: &BestTables,
    table: Which,
    which: rc2k_core::menu::standings::Numbering,
    number: fn(&mut rc2k_core::menu::player::Player) -> &mut u32,
) -> Result<Numbered, Refused> {
    let walked: Vec<(u32, u32)> = match table {
        Which::A => walked_entries_a(tables)?
            .iter()
            .map(|e| (e.head.group, e.head.player))
            .collect(),
        Which::B => walked_entries_b(tables)?
            .iter()
            .map(|e| (e.group, e.player))
            .collect(),
    };
    for &(group, p) in &walked {
        if which.counts(group) && p > 4 {
            return Err(BadInput("a standings player past 4").into());
        }
    }
    let groups: Vec<u32> = walked.iter().map(|&(g, _)| g).collect();
    let numbers = rc2k_core::menu::standings::number(&groups, which);
    let mut last_number = 0;
    for (&(_, p), n) in walked.iter().zip(&numbers) {
        let Some(n) = *n else {
            continue;
        };
        last_number = n;
        if p != 0 {
            let record = player_in(play, p - 1);
            record.update(|q| *number(q) = n);
        }
    }
    let set = |n: usize| numbers[n];
    match table {
        Which::A => with_a(tables, |t| {
            for (n, e) in t.entries[..walked.len()].iter_mut().enumerate() {
                if let Some(v) = set(n) {
                    e.head.figure = v;
                }
            }
        })?,
        Which::B => with_b(tables, |t| {
            for (n, e) in t.entries[..walked.len()].iter_mut().enumerate() {
                if let Some(v) = set(n) {
                    e.position = v;
                }
            }
        })?,
    }
    Ok(Numbered { last_number })
}
