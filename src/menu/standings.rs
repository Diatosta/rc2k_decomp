//! The championship's standings tables (a reading) as the event code walks them: table A
//! at `dword_7556EC` (entries of 0x9C: +4 the player, 1 .. 4 or 0 for a computer driver;
//! +0x1C the group; +0x44 a figure), table B at `dword_7556F0` (entries of 0x78: +4 the
//! player, +0x10 the group, +0x70 a figure); both end with an entry whose first dword is -1.
//! Also the drivers (`g_drivers`, 0x94 each, -1 at the end) and the players
//! (`g_players`, four pointers).

use crate::errors::{BadInput, FatalError, Refused};
use crate::memory::blocks::{At, BlockVar, Place};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::state_store::MENU_STATE;
use crate::session::Dword;
use rc2k_core::menu::standings::{self, Entries, NAME_LEN};

pub const TABLE_A_RVA: &BlockVar = &CHAMP_VIEW.ranked_times; // dword_7556EC
pub const TABLE_B_RVA: &BlockVar = &CHAMP_VIEW.champ_history; // dword_7556F0
pub const A_ENTRY: u32 = 0x9C;
pub const B_ENTRY: u32 = 0x78;
pub const DRIVER: u32 = 0x94;
/// No table of the game's is longer; a walk past it (no end entry) is refused.
pub const MOST_ENTRIES: u32 = 0x400;
/// "Any group" for the first-player searches.
pub const ANY_GROUP: u32 = standings::ANY_GROUP;

pub fn get(p: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(p).read::<u32>("a game value")
}

/// Whether any of the players has no name (null +0x18); the first one found ends the look.
pub fn any_unnamed(players: &[u32]) -> Result<bool, crate::errors::BadInput> {
    for &p in players {
        if get(p + 0x18)? == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn put(p: u32, v: u32) -> Result<(), crate::errors::BadInput> {
    crate::pointers::GamePtr::new(p).write(v, "a game value")
}

/// The dword at a place (a table's, or the game's).
pub fn get_at(p: Place) -> Result<u32, BadInput> {
    p.read("a table's value")
}

/// Player `n`'s record.
pub fn player(n: u32) -> u32 {
    PLAYER_TABLE.pointers[n as usize].get()
}

/// The entries from `first` on, `size` apart, as the original's `do .. while` walks them
/// (the first always, then each while its first dword is not -1); refused when no end
/// entry comes within `MOST_ENTRIES`.
pub fn entries(first: Place, size: u32) -> Result<Vec<Place>, BadInput> {
    let mut out = vec![first];
    let mut e = first;
    loop {
        e = e.add(size);
        if get_at(e)? == 0xFFFF_FFFF {
            return Ok(out);
        }
        if out.len() as u32 >= MOST_ENTRIES {
            return Err(BadInput("a standings table without its end entry"));
        }
        out.push(e);
    }
}

/// `entries` over a table of the game's memory (the drivers): their addresses.
pub fn game_entries(first: u32, size: u32) -> Result<Vec<u32>, BadInput> {
    let count = entries(Place::Game(first), size)?.len() as u32;
    Ok((0..count).map(|n| first.wrapping_add(n * size)).collect())
}

/// The original's search for the first player (`find1stplayer`): from `from` (edi) on, the
/// entries of `group` (ebx) (`ANY_GROUP`: any entry whose group is not 0) whose player (+4) is not 0;
/// ecx counts the group's computer drivers before it, edx its figure, edi the entry. None:
/// the fatal error `code` with `message`.
pub fn first_player(
    from: Place,
    group: u32,
    size: u32,
    group_at: u32,
    figure_at: u32,
    code: u32,
    message: &'static str,
) -> Result<Found, Refused> {
    // Converted: `rc2k_core::menu::standings::first_player`, over the entries walked.
    let walked = entries(from, size)?;
    let found = with_table(from, size, walked.len() as u32, |t| {
        rc2k_core::menu::standings::first_player(t, group, group_at as usize, figure_at as usize)
    })?;
    match found {
        Some(f) => Ok(Found {
            computers: f.computers,
            figure: f.figure,
            entry: walked[f.entry],
        }),
        None => Err(FatalError { code, message }.into()),
    }
}

/// What the first-player searches find, as the original leaves it in the registers: ecx
/// (`computers`, the group's computer drivers before it; the player's number for
/// `first_player_unplaced`), edx its figure, edi the entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub computers: u32,
    pub figure: u32,
    pub entry: Place,
}

impl Found {
    /// The carried values as the original leaves them (ecx the screens' `measure_x`, edx
    /// their `clock`; edi, the entry, is nothing a screen reads).
    pub fn carry_on(&self, c: &mut crate::menu::screen_loop::Carried) {
        c.measure_x = self.computers;
        c.clock = self.figure;
    }
}

/// A table's bytes, `len` from `first`: refused unless they all lie in one block of the
/// game's memory (the table is used by this one caller meanwhile, as the original's code).
pub fn table_bytes<'a>(first: u32, len: u32) -> Result<&'a mut [u8], BadInput> {
    crate::menu::event_flow::game_slice(first, len, "a standings table")
}

/// `f` on `count` entries of `size` bytes from `first`, for the core; refused when they do
/// not lie in its block.
pub fn with_table<R>(
    first: Place,
    size: u32,
    count: u32,
    f: impl FnOnce(&mut Entries) -> R,
) -> Result<R, BadInput> {
    first.with_bytes((count * size) as usize, "a standings table", |bytes| {
        f(&mut Entries {
            bytes,
            size: size as usize,
        })
    })
}

/// `count` entries of `size` bytes from `first`, for the core; refused when they do not lie
/// in one block of the game's memory.
pub fn table<'a>(first: u32, size: u32, count: u32) -> Result<Entries<'a>, BadInput> {
    Ok(Entries {
        bytes: table_bytes(first, count * size)?,
        size: size as usize,
    })
}

/// The drivers (`g_drivers`) as the core's table, as far as `entries` walks them.
pub fn driver_table<'a>() -> Result<Entries<'a>, BadInput> {
    let first = crate::menu::drivers_store::DRIVERS.first();
    let all = game_entries(first, DRIVER)?;
    table(first, DRIVER, all.len() as u32)
}

/// A record's stores (the core's) made at `p`.
pub fn put_all(p: u32, stores: &[standings::Store]) -> Result<(), BadInput> {
    for &(at, v) in stores {
        put(p + at as u32, v)?;
    }
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
pub fn counted_players() -> Result<Vec<u32>, BadInput> {
    let n: u32 = EVENT_STATE.player_count.get();
    if n == 0 || n > 4 || (0..n).any(|k| player(k) == 0) {
        return Err(BadInput(
            "a player count of 0 or past 4, or a player missing",
        ));
    }
    Ok((0..n).map(player).collect())
}

/// The current record's event (`g_recordTable`, `section`, `index`; its +0x3C) in
/// `dword_7556D8`'s table, refused past the table's sections.
pub fn current_event() -> Result<At, BadInput> {
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
    let events = CHAMP_VIEW.stage_best_times.need("no events table")?;
    Ok(events.add(get(record + 0x3C)?.wrapping_mul(0x618)))
}

/// Group `group`'s tenth best score (`saves::best_times::SCORES`); refused with no table or
/// a group past the five.
pub fn tenth_best_score(group: u32) -> Result<u32, BadInput> {
    crate::saves::best_times::SCORES
        .with(|t| t.groups.get(group as usize).map(|g| g[9].score))?
        .ok_or(BadInput("a group past the best scores' five"))
}
const SORTED: u32 = standings::SORTED as u32;

/// Standings table A's first 0xB4 entries sorted by the signed dword at `key` (the
/// original's exchange sort: an entry later or equal moves up; `dword_7559B4` /
/// `dword_7559B8` its counters). Refused without a table.
pub fn sort_table_a(key: u32) -> Result<(), BadInput> {
    let first = CHAMP_VIEW.ranked_times.need("no standings table")?;
    // Converted: `rc2k_core::menu::standings::exchange_sort`.
    with_table(Place::Owned(first), A_ENTRY, SORTED, |t| {
        with_kept_name(|kept| {
            standings::exchange_sort(
                t,
                key as usize,
                false,
                &standings::A_FIELDS,
                standings::A_NAME,
                kept,
            )
        })
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
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(n);
    let group = group.unwrap_or(get(p + 0x90)?);
    let scores = &crate::saves::best_times::SCORES;
    let (true, true, false) = (scores.is_taken(), group < 5, get(p + 0x18)? == 0) else {
        return Err(BadInput("no best-score table, a group past 4 or no name"));
    };
    // Converted: `rc2k_core::saves::scores::enter_best`.
    use rc2k_core::saves::scores as s;
    let name_ref = get(p + 0x18)?;
    let best = s::Best {
        player: n + 1,
        score: get(p + score_at)?,
        nation: get(p + 4)?,
        car: get(p + 8)?,
        group: get(p + 0x90)?,
        name_ref,
        name: bytes(name_ref)?,
    };
    scores.with(|t| {
        with_kept_name(|kept| s::enter_best(&mut t.groups[group as usize], &best, kept))
    })?;
    Ok(())
}

/// Standings table B's first 0xB4 entries sorted by the signed dword at `key`, as
/// `sort_table_a`: going up (a later entry not above an earlier one moves up) or, with
/// `down`, going down (a later entry not below one moves up). Refused without a table.
pub fn sort_table_b(key: u32, down: bool) -> Result<(), BadInput> {
    let first = CHAMP_VIEW.champ_history.need("no standings table")?;
    // Converted: `rc2k_core::menu::standings::exchange_sort`.
    with_table(Place::Owned(first), B_ENTRY, SORTED, |t| {
        with_kept_name(|kept| {
            standings::exchange_sort(
                t,
                key as usize,
                down,
                &standings::b_fields(),
                standings::B_NAME,
                kept,
            )
        })
    })?;

    DRAW_WORK.work_b4.set(SORTED - 1);
    DRAW_WORK.work_b8.set(SORTED);

    Ok(())
}

/// Whether the player's part `k` (an `INSPECTED` index) is at or past its limit (signed).
pub fn past_limit(p: u32, k: u32) -> Result<bool, BadInput> {
    let limit = MENU_STATE.driver_stats.limits[k as usize].get() as i32;
    Ok(get(p + 0xD4 + 4 * k)? as i32 >= limit)
}

/// What the numbering functions (`number_all_a`, `number_all_b`, `number_standings_a`,
/// `number_standings_b`) leave in the registers, which their callers' later calls may read:
/// the last number given (ecx), the last numbered player as (index, record) when there was
/// one (eax, esi; else they are kept). (The original's edi, the table's end, is read by no
/// later call: not kept since the tables are blocks, phase 12.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Numbered {
    pub last_number: u32,
    pub last_player: Option<(u32, u32)>,
}

impl Numbered {
    /// The carried values as the original leaves them (eax, ecx, esi; edi is nothing a
    /// screen reads).
    pub fn carry_on(&self, c: &mut crate::menu::screen_loop::Carried) {
        c.measure_x = self.last_number;
        if let Some((index, record)) = self.last_player {
            c.arg = index;
            c.cars = record;
        }
    }
}
