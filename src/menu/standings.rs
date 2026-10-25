//! The championship's standings tables (a reading) as the event code walks them: table A
//! at `dword_7556EC` (entries of 0x9C: +4 the player, 1 .. 4 or 0 for a computer driver;
//! +0x1C the group; +0x44 a figure), table B at `dword_7556F0` (entries of 0x78: +4 the
//! player, +0x10 the group, +0x70 a figure); both end with an entry whose first dword is -1.
//! Also the drivers (`g_drivers`, 0x94 each, -1 at the end) and the players
//! (`g_players`, four pointers).

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::state_store::MENU_STATE;
use crate::session::Dword;
use rc2k_core::menu::standings::{self, Entries, NAME_LEN};

pub const TABLE_A_RVA: &Dword = &CHAMP_VIEW.ranked_times; // dword_7556EC
pub const TABLE_B_RVA: &Dword = &CHAMP_VIEW.champ_history; // dword_7556F0
pub const A_ENTRY: u32 = 0x9C;
pub const B_ENTRY: u32 = 0x78;
pub const DRIVER: u32 = 0x94;
/// No table of the game's is longer; a walk past it (no end entry) is refused.
pub const MOST_ENTRIES: u32 = 0x400;
/// "Any group" for the first-player searches.
pub const ANY_GROUP: u32 = standings::ANY_GROUP;

pub fn get(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

pub fn put(p: u32, v: u32) {
    unsafe { (p as *mut u32).write_unaligned(v) };
}

/// Player `n`'s record.
pub fn player(n: u32) -> u32 {
    PLAYER_TABLE.pointers[n as usize].get()
}

/// The entries from `first` on, `size` apart, as the original's `do .. while` walks them
/// (the first always, then each while its first dword is not -1); refused when no end
/// entry comes within `MOST_ENTRIES`.
pub fn entries(first: u32, size: u32) -> Result<Vec<u32>, BadInput> {
    let mut out = vec![first];
    let mut e = first;
    loop {
        e = e.wrapping_add(size);
        if get(e) == 0xFFFF_FFFF {
            return Ok(out);
        }
        if out.len() as u32 >= MOST_ENTRIES {
            return Err(BadInput("a standings table without its end entry"));
        }
        out.push(e);
    }
}

/// The original's search for the first player (`find1stplayer`): from `from` (edi) on, the
/// entries of `group` (ebx) (`ANY_GROUP`: any entry whose group is not 0) whose player (+4) is not 0;
/// ecx counts the group's computer drivers before it, edx its figure, edi the entry. None:
/// the fatal error `code` with `message`.
pub fn first_player(
    from: u32,
    group: u32,
    size: u32,
    group_at: u32,
    figure_at: u32,
    code: u32,
    message: &'static str,
) -> Result<Found, Refused> {
    // Converted: `rc2k_core::menu::standings::first_player`, over the entries walked.
    let walked = entries(from, size)?;
    let bytes = unsafe { table_bytes(from, walked.len() as u32 * size) };
    let t = Entries {
        bytes,
        size: size as usize,
    };
    match rc2k_core::menu::standings::first_player(&t, group, group_at as usize, figure_at as usize)
    {
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
    pub entry: u32,
}

impl Found {
    /// The carried values as the original leaves them (ecx the screens' `measure_x`, edx
    /// their `clock`; edi, the entry, is nothing a screen reads).
    pub fn carry_on(&self, c: &mut crate::menu::screen_loop::Carried) {
        c.measure_x = self.computers;
        c.clock = self.figure;
    }
}

/// A table's bytes, `len` from `first`.
///
/// # Safety
/// `first` .. `first + len` must be game memory no one else uses meanwhile.
pub unsafe fn table_bytes<'a>(first: u32, len: u32) -> &'a mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(first as *mut u8, len as usize) }
}

/// `count` entries of `size` bytes from `first`, for the core.
///
/// # Safety
/// As `table_bytes`.
pub unsafe fn table<'a>(first: u32, size: u32, count: u32) -> Entries<'a> {
    Entries {
        bytes: unsafe { table_bytes(first, count * size) },
        size: size as usize,
    }
}

/// The drivers (`g_drivers`) as the core's table, as far as `entries` walks them.
pub fn driver_table<'a>() -> Result<Entries<'a>, BadInput> {
    let first = crate::menu::drivers_store::DRIVERS.first();
    let all = entries(first, DRIVER)?;
    Ok(unsafe { table(first, DRIVER, all.len() as u32) })
}

/// A record's stores (the core's) made at `p`.
pub fn put_all(p: u32, stores: &[standings::Store]) {
    for &(at, v) in stores {
        put(p + at as u32, v);
    }
}

/// `N` bytes at `p`.
pub fn bytes<const N: usize>(p: u32) -> [u8; N] {
    unsafe { (p as *const [u8; N]).read_unaligned() }
}

/// `g_keptPlayerName`, the sorts' swap buffer.
pub fn kept_name<'a>() -> &'a mut [u8; NAME_LEN] {
    unsafe { &mut *(NAME_BUFFERS.kept_name.addr() as *mut [u8; NAME_LEN]) }
}

pub const PLAYER_COUNT_RVA: &Dword = &EVENT_STATE.player_count; // dword_755984
pub const SECTION_RVA: &Dword = &EVENT_STATE.section; // section
pub const INDEX_RVA: &Dword = &EVENT_STATE.index; // index
pub const EVENTS_RVA: &Dword = &CHAMP_VIEW.stage_best_times; // dword_7556D8: 0x618 an event

/// The players `0 .. dword_755984` (the original's `do .. while`): refused at 0, past 4 or
/// with one missing.
pub fn counted_players() -> Result<Vec<u32>, BadInput> {
    let n: u32 = { EVENT_STATE.player_count.get() };
    if n == 0 || n > 4 || (0..n).any(|k| player(k) == 0) {
        return Err(BadInput(
            "a player count of 0 or past 4, or a player missing",
        ));
    }
    Ok((0..n).map(player).collect())
}

/// The current record's event (`g_recordTable`, `section`, `index`; its +0x3C) in
/// `dword_7556D8`'s table, refused past the table's sections.
pub fn current_event() -> Result<u32, BadInput> {
    let table = crate::records::stages_store::STAGE_TABLES.championship();
    let section: u32 = { EVENT_STATE.section.get() };
    let index: u32 = { EVENT_STATE.index.get() };
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(table, section, 600)
    {
        return Err(BadInput("a record past the table"));
    }
    let record = crate::records::find_record::find_record_in(table, index as i32, section);
    let events: u32 = { CHAMP_VIEW.stage_best_times.get() };
    if events == 0 {
        return Err(BadInput("no events table"));
    }
    Ok(events.wrapping_add(get(record + 0x3C).wrapping_mul(0x618)))
}

pub const BEST_SCORES_RVA: &Dword = &CHAMP_VIEW.best_scores; // dword_7556D4: five groups of ten, 0x3C each
const SORTED: u32 = standings::SORTED as u32;

/// Standings table A's first 0xB4 entries sorted by the signed dword at `key` (the
/// original's exchange sort: an entry later or equal moves up; `dword_7559B4` /
/// `dword_7559B8` its counters). Refused without a table.
pub fn sort_table_a(key: u32) -> Result<(), BadInput> {
    let first: u32 = { CHAMP_VIEW.ranked_times.get() };
    if first == 0 {
        return Err(BadInput("no standings table"));
    }
    // Converted: `rc2k_core::menu::standings::exchange_sort`.
    let mut t = Entries {
        bytes: unsafe { table_bytes(first, SORTED * A_ENTRY) },
        size: A_ENTRY as usize,
    };
    standings::exchange_sort(
        &mut t,
        key as usize,
        false,
        &standings::A_FIELDS,
        standings::A_NAME,
        kept_name(),
    );

    DRAW_WORK.work_b4.set(SORTED - 1);
    DRAW_WORK.work_b8.set(SORTED);

    Ok(())
}

/// The current player's (`dword_75598C`) best score entered at the bottom of `group`'s ten
/// (`dword_7556D4`, entry 9): +0x38 its score (the record's `score_at`), +4 the player and
/// 1, +0x14 1, +8 / +0xC its +4 / +8, +0x10 its group (+0x90), +0 its name pointer and
/// +0x18 the name's 0x1F bytes; then moved up its group. Refused for a current player past
/// 3 or missing, no name, no table or a group past 4.
pub fn enter_best(group: Option<u32>, score_at: u32) -> Result<(), BadInput> {
    let n: u32 = { EVENT_STATE.current_player.get() };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(n);
    let group = group.unwrap_or(get(p + 0x90));
    let table: u32 = { CHAMP_VIEW.best_scores.get() };
    if table == 0 || group >= 5 || get(p + 0x18) == 0 {
        return Err(BadInput("no best-score table, a group past 4 or no name"));
    }
    // Converted: `rc2k_core::menu::standings::enter_best`.
    let name_at = get(p + 0x18);
    let best = standings::Best {
        player: n + 1,
        score: get(p + score_at),
        car: get(p + 4),
        field_8: get(p + 8),
        group: get(p + 0x90),
        name_at,
        name: unsafe { (name_at as *const [u8; NAME_LEN]).read_unaligned() },
    };
    let mut ten = Entries {
        bytes: unsafe { table_bytes(table + group * 0x258, 0x258) },
        size: 0x3C,
    };
    standings::enter_best(&mut ten, &best, kept_name());
    Ok(())
}

/// Standings table B's first 0xB4 entries sorted by the signed dword at `key`, as
/// `sort_table_a`: going up (a later entry not above an earlier one moves up) or, with
/// `down`, going down (a later entry not below one moves up). Refused without a table.
pub fn sort_table_b(key: u32, down: bool) -> Result<(), BadInput> {
    let first: u32 = { CHAMP_VIEW.champ_history.get() };
    if first == 0 {
        return Err(BadInput("no standings table"));
    }
    // Converted: `rc2k_core::menu::standings::exchange_sort`.
    let mut t = Entries {
        bytes: unsafe { table_bytes(first, SORTED * B_ENTRY) },
        size: B_ENTRY as usize,
    };
    standings::exchange_sort(
        &mut t,
        key as usize,
        down,
        &standings::b_fields(),
        standings::B_NAME,
        kept_name(),
    );

    DRAW_WORK.work_b4.set(SORTED - 1);
    DRAW_WORK.work_b8.set(SORTED);

    Ok(())
}

/// Whether the player's part `k` (an `INSPECTED` index) is at or past its limit (signed).
pub fn past_limit(p: u32, k: u32) -> bool {
    let limit = MENU_STATE.driver_stats.limits[k as usize].get() as i32;
    get(p + 0xD4 + 4 * k) as i32 >= limit
}

/// What the numbering functions (`number_all_a`, `number_all_b`, `number_standings_a`,
/// `number_standings_b`) leave in the registers, which their callers' later calls may read:
/// the last number given (ecx), the last numbered player as (index, record) when there was
/// one (eax, esi; else they are kept), the end of the table (edi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Numbered {
    pub last_number: u32,
    pub last_player: Option<(u32, u32)>,
    pub end: u32,
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
