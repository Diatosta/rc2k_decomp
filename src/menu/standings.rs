//! The championship's standings tables as the event code walks them: table A
//! (`saves::best_times::RANKED_TIMES`, best times: `player` 1 .. 4 or 0 for a computer
//! driver, `group`, `figure` the position) and table B (`saves::best_times::CHAMP_HISTORY`,
//! `rankings::HistoryEntry`); both end with an entry whose first dword is -1.
//! Also the drivers (`g_drivers`, -1 at the end) and the players (`g_players`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::drivers_store::{DRIVERS, Driver};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::state_store::MENU_STATE;
use crate::session::Dword;
use rc2k_core::menu::standings::{self, NAME_LEN};

pub use crate::saves::best_times::{CHAMP_HISTORY as TABLE_B, RANKED_TIMES as TABLE_A};
use rc2k_core::saves::rankings::{BKey, ChampHistory, HistoryEntry, RankedTimes};
/// "Any group" for the first-player searches.
pub const ANY_GROUP: u32 = standings::ANY_GROUP;

pub use crate::menu::event_flow::dword as get;

/// A player's record, typed.
pub type PlayerCell = &'static crate::session::Shared<rc2k_core::menu::player::Player>;

/// Whether any of the players has no name (a null name place).
pub fn any_unnamed(players: &[PlayerCell]) -> Result<bool, crate::errors::BadInput> {
    Ok(players.iter().any(|p| p.get().name == 0))
}

/// Player `n`'s record (`n` one of the four).
pub fn player(n: u32) -> PlayerCell {
    &PLAYER_TABLE.players[n as usize]
}

/// The drivers (`g_drivers`) as the original's `do .. while` walks them: the first always,
/// then each while its first dword is not -1. The walk ends at the latest on the -1 after the
/// 180 (`DRIVERS.end`); refused when that is gone (the original would read on).
pub fn drivers() -> Result<&'static [Driver], BadInput> {
    let all = &DRIVERS.drivers;
    let count = 1 + all[1..]
        .iter()
        .take_while(|d| d.name_entry.get() != u32::MAX)
        .count();
    if count == all.len() && DRIVERS.end.get() != u32::MAX {
        return Err(BadInput("the drivers without their end entry"));
    }
    Ok(&all[..count])
}

/// `f` on table A; refused without it.
pub fn with_a<R>(f: impl FnOnce(&mut RankedTimes) -> R) -> Result<R, BadInput> {
    TABLE_A.with(f)
}

/// `f` on table B; refused without it.
pub fn with_b<R>(f: impl FnOnce(&mut ChampHistory) -> R) -> Result<R, BadInput> {
    TABLE_B.with(f)
}

/// How many of table A's entries the original walks; refused without the table or with no
/// end entry within the core's `MOST_WALKED`.
pub fn walked_a() -> Result<usize, BadInput> {
    with_a(|t| t.walked())?.ok_or(BadInput("a standings table without its end entry"))
}

/// As `walked_a`, of table B.
pub fn walked_b() -> Result<usize, BadInput> {
    with_b(|t| t.walked())?.ok_or(BadInput("a standings table without its end entry"))
}

/// The walked entries of table A, copied out.
pub fn walked_entries_a() -> Result<Vec<rc2k_core::saves::entries::BestTime>, BadInput> {
    let n = walked_a()?;
    with_a(|t| t.entries[..n].to_vec())
}

/// The walked entries of table B, copied out.
pub fn walked_entries_b() -> Result<Vec<HistoryEntry>, BadInput> {
    let n = walked_b()?;
    with_b(|t| t.entries[..n].to_vec())
}

/// Which standings table a search walks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    A,
    B,
}

/// The original's search for the first player (`find1stplayer`) from the table's start:
/// the entries of `group` (ebx) (`ANY_GROUP`: any entry whose group is not 0) whose player
/// is not 0; ecx counts the group's computer drivers before it, edx its figure (the
/// position), edi the entry. None: the fatal error `code` with `message`.
pub fn first_player(
    which: Which,
    group: u32,
    code: u32,
    message: &'static str,
) -> Result<Found, Refused> {
    // Converted: `rc2k_core::saves::rankings::first_player`, over the entries walked.
    use rc2k_core::saves::rankings::first_player as find;
    let found = match which {
        Which::A => find(&walked_entries_a()?, group, |e| {
            (e.head.group, e.head.player, e.head.figure)
        }),
        Which::B => find(&walked_entries_b()?, group, |e| {
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

/// What the first-player searches find, as the original leaves it in the registers: ecx
/// (`computers`, the group's computer drivers before it; the player's number for
/// `first_player_unplaced`), edx its figure, edi the entry (here its number in the table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub computers: u32,
    pub figure: u32,
    pub entry: usize,
}

/// The walked drivers, for the tables' fills.
pub fn walked_drivers() -> Result<Vec<rc2k_core::saves::rankings::Driver>, BadInput> {
    Ok(drivers()?
        .iter()
        .map(|d| rc2k_core::saves::rankings::Driver {
            number: d.name_entry.get(),
            group: d.class.get(),
            time: d.time.get(),
            total: d.total.get(),
            nation: d.nation.get(),
            car: d.car.get(),
            entered: d.in_event.get(),
            points: d.results.each_ref().map(Dword::get),
        })
        .collect())
}

/// A record copied out of the game's memory for the core to work on (a stage record's,
/// `menu::keep_stage_best`), stored back when dropped
/// (whatever the caller's way out: the original works in place, so its changes stay).
pub struct GameTable {
    first: u32,
    size: usize,
    bytes: Vec<u8>,
}

impl GameTable {
    /// The core's view.
    pub fn entries(&mut self) -> rc2k_core::menu::standings::Entries<'_> {
        rc2k_core::menu::standings::Entries {
            bytes: &mut self.bytes,
            size: self.size,
        }
    }
}

impl Drop for GameTable {
    fn drop(&mut self) {
        crate::pointers::GamePtr::new(self.first)
            .write_bytes(&self.bytes, "a standings table")
            .unwrap_or_else(|e| crate::fatal::raise::end("a standings table", e));
    }
}

/// `count` entries of `size` bytes from `first`, for the core; refused when they do not lie
/// in one block of the game's memory.
pub fn table(first: u32, size: u32, count: u32) -> Result<GameTable, BadInput> {
    Ok(GameTable {
        first,
        size: size as usize,
        bytes: crate::pointers::GamePtr::new(first)
            .read_bytes((count * size) as usize, "a standings table")?,
    })
}

/// A record's stores (the core's) made at `p`.
pub fn put_all(p: PlayerCell, stores: &[standings::Store]) -> Result<(), BadInput> {
    p.update(|p| p.store(stores));
    Ok(())
}

/// `N` bytes at `p`.
pub fn bytes<const N: usize>(p: u32) -> Result<[u8; N], BadInput> {
    crate::pointers::GamePtr::new(p).read::<[u8; N]>("a name")
}

/// Runs `f` on `g_keptPlayerName`, the sorts' swap buffer (a copy of it, put back after).
pub fn with_kept_name<R>(f: impl FnOnce(&mut [u8; NAME_LEN]) -> R) -> R {
    let mut all = NAME_BUFFERS.kept_name.read();
    let [kept @ .., _] = &mut all;
    let r = f(kept);
    NAME_BUFFERS.kept_name.write(&all);
    r
}

pub const PLAYER_COUNT_RVA: &Dword = &EVENT_STATE.player_count; // dword_755984
pub const SECTION_RVA: &Dword = &EVENT_STATE.section; // section
pub const INDEX_RVA: &Dword = &EVENT_STATE.index; // index

/// The players `0 .. dword_755984` (the original's `do .. while`): refused at 0, past 4 or
/// with one missing.
pub fn counted_players() -> Result<Vec<PlayerCell>, BadInput> {
    let n: u32 = EVENT_STATE.player_count.get();
    if n == 0 || n > 4 {
        return Err(BadInput("a player count of 0 or past 4"));
    }
    Ok((0..n).map(player).collect())
}

/// The current record's event (`g_recordTable`, `section`, `index`; its +0x3C): its place
/// in the stages' best times (`saves::best_times::STAGE_TIMES`); refused past the table's
/// sections, with no table, or past it.
pub fn current_event() -> Result<usize, BadInput> {
    let table = crate::records::stages_store::STAGE_TABLES.championship();
    let section: u32 = EVENT_STATE.section.get();
    let index: u32 = EVENT_STATE.index.get();
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(table, section, 600)?
    {
        return Err(BadInput("a record past the table"));
    }
    let record = crate::records::find_record::find_record_in(table, index as i32, section)?;
    let event = get(record + 0x3C)? as usize;
    if !crate::saves::best_times::STAGE_TIMES.with(|t| event < t.stages.len())? {
        return Err(BadInput("an event past the stages' best times"));
    }
    Ok(event)
}

/// `f` on event `event`'s ten record entries (`current_event`'s place).
pub fn with_event<R>(
    event: usize,
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, BadInput> {
    crate::saves::best_times::STAGE_TIMES
        .with(|t| t.stages.get_mut(event).map(f))?
        .ok_or(BadInput("an event past the stages' best times"))
}

/// Group `group`'s tenth best score (`saves::best_times::SCORES`); refused with no table or
/// a group past the five.
pub fn tenth_best_score(group: u32) -> Result<u32, BadInput> {
    crate::saves::best_times::SCORES
        .with(|t| t.groups.get(group as usize).map(|g| g[9].score))?
        .ok_or(BadInput("a group past the best scores' five"))
}
const SORTED: u32 = standings::SORTED as u32;

/// Standings table A's first 0xB4 entries sorted by `key` (signed; the original's
/// exchange sort: an entry later or equal moves up; `dword_7559B4` / `dword_7559B8` its
/// counters). Refused without a table.
pub fn sort_table_a(key: rc2k_core::saves::entries::Field) -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::rankings::sort_a`.
    with_a(|t| {
        with_kept_name(|kept| rc2k_core::saves::rankings::sort_a(t, SORTED as usize, key, kept));
    })?;

    DRAW_WORK.work_b4.set(SORTED - 1);
    DRAW_WORK.work_b8.set(SORTED);

    Ok(())
}

/// The current player's (`dword_75598C`) best score entered at the bottom of `group`'s ten
/// (`saves::best_times::SCORES`, entry 9): its score (the record's `score_at`), its number
/// plus 1, `new` 1, the nationality and car (its +4 / +8), its group (+0x90), its name pointer
/// and the name's 0x1F bytes; then moved up its group. Refused for a current player past
/// 3 or missing, no name, no table or a group past 4.
pub fn enter_best(group: Option<u32>, score_at: u32) -> Result<(), BadInput> {
    let n: u32 = EVENT_STATE.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(n);
    let group = group.unwrap_or(p.get().word(0x90));
    let scores = &crate::saves::best_times::SCORES;
    let (true, true, false) = (scores.is_taken(), group < 5, p.get().word(0x18) == 0) else {
        return Err(BadInput("no best-score table, a group past 4 or no name"));
    };
    // Converted: `rc2k_core::saves::scores::enter_best`.
    use rc2k_core::saves::scores as s;
    let name_ref = p.get().word(0x18);
    let best = s::Best {
        player: n + 1,
        score: p.get().word(score_at as usize),
        nation: p.get().word(4),
        car: p.get().word(8),
        group: p.get().word(0x90),
        name_ref,
        name: bytes(name_ref)?,
    };
    scores.with(|t| {
        with_kept_name(|kept| s::enter_best(&mut t.groups[group as usize], &best, kept));
    })?;
    Ok(())
}

/// Standings table B's first 0xB4 entries sorted by `key`, as `sort_table_a`: going up (a
/// later entry not above an earlier one moves up) or, with `down`, going down (a later
/// entry not below one moves up). Refused without a table.
pub fn sort_table_b(key: BKey, down: bool) -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::rankings::sort_b`.
    with_b(|t| {
        with_kept_name(|kept| {
            rc2k_core::saves::rankings::sort_b(t, SORTED as usize, key, down, kept);
        });
    })?;

    DRAW_WORK.work_b4.set(SORTED - 1);
    DRAW_WORK.work_b8.set(SORTED);

    Ok(())
}

/// Whether the player's part `k` (an `INSPECTED` index) is at or past its limit (signed).
pub fn past_limit(p: PlayerCell, k: u32) -> Result<bool, BadInput> {
    let limit = MENU_STATE.driver_stats.limits[k as usize].get() as i32;
    Ok(p.get().word(0xD4 + 4 * k as usize) as i32 >= limit)
}

/// What the numbering functions (`number_all_a`, `number_all_b`, `number_standings_a`,
/// `number_standings_b`) give: the last number given (the class passes store it).
#[derive(Clone, Copy)]
pub struct Numbered {
    pub last_number: u32,
}

/// The numbering of a standings table (`number_all_*`, `number_standings_*`): each entry
/// `which` counts numbered in turn, its position set and, for a player's, the player's
/// record's dword at `record_at`. Refused, before any write, when a counted entry's player
/// is past 4 or not there.
pub fn number_table(
    table: Which,
    which: rc2k_core::menu::standings::Numbering,
    record_at: u32,
) -> Result<Numbered, Refused> {
    let walked: Vec<(u32, u32)> = match table {
        Which::A => walked_entries_a()?
            .iter()
            .map(|e| (e.head.group, e.head.player))
            .collect(),
        Which::B => walked_entries_b()?
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
            let record = player(p - 1);
            record.update(|q| q.store(&[(record_at as usize, n)]));
        }
    }
    let set = |n: usize| numbers[n];
    match table {
        Which::A => with_a(|t| {
            for (n, e) in t.entries[..walked.len()].iter_mut().enumerate() {
                if let Some(v) = set(n) {
                    e.head.figure = v;
                }
            }
        })?,
        Which::B => with_b(|t| {
            for (n, e) in t.entries[..walked.len()].iter_mut().enumerate() {
                if let Some(v) = set(n) {
                    e.position = v;
                }
            }
        })?,
    }
    Ok(Numbered { last_number })
}
