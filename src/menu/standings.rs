//! The championship's standings tables (a reading) as the event code walks them: table A
//! at `dword_7556EC` (entries of 0x9C: +4 the player, 1 .. 4 or 0 for a computer driver;
//! +0x1C the group; +0x44 a figure), table B at `dword_7556F0` (entries of 0x78: +4 the
//! player, +0x10 the group, +0x70 a figure); both end with an entry whose first dword is -1.
//! Also the drivers (`g_drivers`, 0x94 each, -1 at the end) and the players
//! (`g_players`, four pointers).

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use rc2k_core::menu::standings::{self, Entries, NAME_LEN};

pub const TABLE_A_RVA: usize = rva(0x7556EC); // dword_7556EC
pub const TABLE_B_RVA: usize = rva(0x7556F0); // dword_7556F0
pub const A_ENTRY: u32 = 0x9C;
pub const B_ENTRY: u32 = 0x78;
pub const DRIVERS_RVA: usize = rva(0x73B674); // g_drivers
pub const DRIVER: u32 = 0x94;
pub const PLAYERS_RVA: usize = rva(0x75647C); // g_players
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
    unsafe { game_read(PLAYERS_RVA + 4 * n as usize) }
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

/// The original's search for the first player (`find1stplayer`): from edi on, the entries
/// of `group` (`ANY_GROUP`: any entry whose group is not 0) whose player (+4) is not 0;
/// ecx counts the group's computer drivers before it, edx its figure, edi the entry. None:
/// the fatal error `code` with `message`.
pub fn first_player(
    entry: &Regs,
    size: u32,
    group_at: u32,
    figure_at: u32,
    code: u32,
    message: &'static str,
) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::standings::first_player`, over the entries walked.
    let walked = entries(entry.edi, size)?;
    let bytes = unsafe { table_bytes(entry.edi, walked.len() as u32 * size) };
    let t = Entries {
        bytes,
        size: size as usize,
    };
    match rc2k_core::menu::standings::first_player(
        &t,
        entry.ebx,
        group_at as usize,
        figure_at as usize,
    ) {
        Some(f) => Ok(Regs {
            ecx: f.computers,
            edx: f.figure,
            edi: walked[f.entry],
            ..*entry
        }),
        None => Err(FatalError { code, message }.into()),
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
    let first = game_addr(DRIVERS_RVA) as u32;
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
    unsafe { &mut *(game_addr(KEPT_NAME_RVA) as *mut [u8; NAME_LEN]) }
}

/// Test entries: `n` of `size` bytes from `p` (figures by lot, groups 1 .. 3 at `group_at`,
/// a player at some), then the end entry.
pub fn fill_test_table(p: u32, size: u32, n: u32, group_at: u32, case: u32) {
    for k in 0..n {
        let e = p + k * size;
        for o in (0..size).step_by(4) {
            put(e + o, (k * 0x3B + o * 7 + case * 0x101) & 0xFF);
        }
        put(e, k);
        put(
            e + 4,
            if (k * 5 + case).is_multiple_of(4) {
                1 + k % 4
            } else {
                0
            },
        );
        put(e + group_at, 1 + (k + case) % 3);
    }
    put(p + n * size, 0xFFFF_FFFF);
}

/// The test of a clearing hook: a table of the test's (eight entries) in `table`.
pub fn test_clear(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    table: usize,
    size: u32,
    rust: &dyn Fn(&Regs) -> Result<crate::screens::harness::Left, Refused>,
) -> (usize, usize) {
    use crate::framework::{SelfTest, game_write};
    use crate::service::harness::{at, capture, check_state, restore};
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut buf = vec![0u32; (9 * size / 4) as usize];
    let tp = buf.as_mut_ptr() as u32;
    let regions = vec![(tp, (9 * size) as usize)];
    let vars = vec![at(table)];
    let saved = capture(&vars);
    for case in 0..3u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(table, tp);
                fill_test_table(tp, size, 8, 0x10, case);
            },
            &regions,
            rust,
        );
    }
    restore(&vars, &saved);
    drop(buf);
    t.finish()
}

/// The test of a first-player search: a table of the test's from edi, groups by lot; where
/// the group has no player the Rust version's fatal error is checked (the original's would
/// end the game).
#[allow(clippy::too_many_arguments)]
pub fn test_first_player(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    size: u32,
    group_at: u32,
    any_group: bool,
    code: u32,
    rust: &dyn Fn(&Regs) -> Result<crate::screens::harness::Left, Refused>,
) -> (usize, usize) {
    use crate::framework::SelfTest;
    use crate::service::harness::check_state;
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut buf = vec![0u32; (9 * size / 4) as usize];
    let tp = buf.as_mut_ptr() as u32;
    for case in 0..8u32 {
        let group = if any_group {
            [ANY_GROUP, 1, 2, 3][case as usize % 4]
        } else {
            0
        };
        let before = Regs {
            edi: tp,
            ebx: group,
            ..Regs::SENTINEL
        };
        fill_test_table(tp, size, 8, group_at, case);
        let has = (0..8u32).any(|k| {
            let e = tp + k * size;
            let g = get(e + group_at);
            let in_group = if !any_group || group == ANY_GROUP {
                g != 0
            } else {
                g == group
            };
            in_group && get(e + 4) != 0
        });
        if !has {
            let refused = rust(&before);
            t.check(
                matches!(refused, Err(Refused::Fatal(e)) if e.code == code),
                || {
                    format!(
                        "case {case}: group {group:#x} has no player, Rust gave {:?}",
                        refused.as_ref().err()
                    )
                },
            );
            continue;
        }
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            before,
            &|| fill_test_table(tp, size, 8, group_at, case),
            &[],
            rust,
        );
    }
    drop(buf);
    t.finish()
}

pub const PLAYER_COUNT_RVA: usize = rva(0x755984); // dword_755984
pub const SECTION_RVA: usize = rva(0x75595C); // section
pub const INDEX_RVA: usize = rva(0x755958); // index
pub const RECORDS_RVA: usize = rva(0x735576); // g_recordTable
pub const EVENTS_RVA: usize = rva(0x7556D8); // dword_7556D8: 0x618 an event

/// The players `0 .. dword_755984` (the original's `do .. while`): refused at 0, past 4 or
/// with one missing.
pub fn counted_players() -> Result<Vec<u32>, BadInput> {
    let n: u32 = unsafe { game_read(PLAYER_COUNT_RVA) };
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
    let table = game_addr(RECORDS_RVA) as u32;
    let section: u32 = unsafe { game_read(SECTION_RVA) };
    let index: u32 = unsafe { game_read(INDEX_RVA) };
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(table, section, 600)
    {
        return Err(BadInput("a record past the table"));
    }
    let record = crate::records::find_record::find_record_in(table, index as i32, section);
    let events: u32 = unsafe { game_read(EVENTS_RVA) };
    if events == 0 {
        return Err(BadInput("no events table"));
    }
    Ok(events.wrapping_add(get(record + 0x3C).wrapping_mul(0x618)))
}

pub const KEPT_NAME_RVA: usize = rva(0x755884); // g_keptPlayerName: 0x1F bytes
pub const BEST_SCORES_RVA: usize = rva(0x7556D4); // dword_7556D4: five groups of ten, 0x3C each
const SORT_I_RVA: usize = rva(0x7559B4); // dword_7559B4
const SORT_J_RVA: usize = rva(0x7559B8); // dword_7559B8
const SORTED: u32 = standings::SORTED as u32;

/// Standings table A's first 0xB4 entries sorted by the signed dword at `key` (the
/// original's exchange sort: an entry later or equal moves up; `dword_7559B4` /
/// `dword_7559B8` its counters). Refused without a table.
pub fn sort_table_a(key: u32) -> Result<(), BadInput> {
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
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
    unsafe {
        crate::framework::game_write(SORT_I_RVA, SORTED - 1);
        crate::framework::game_write(SORT_J_RVA, SORTED);
    }
    Ok(())
}

/// What `sort_table_a` writes.
pub fn sort_regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    vec![
        (first, (SORTED * A_ENTRY) as usize),
        at(SORT_I_RVA),
        at(SORT_J_RVA),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
    ]
}

/// The last best score of `group` moved up its ten (the original's nine compares from the
/// bottom: a later entry not below the one above it moves up). Refused without a table or
/// past the five groups.
pub fn rank_in_group(group: u32) -> Result<(), BadInput> {
    let table: u32 = unsafe { game_read(BEST_SCORES_RVA) };
    if table == 0 || group >= 5 {
        return Err(BadInput("no best-score table, or a group past 4"));
    }
    // Converted: `rc2k_core::menu::standings::rank_in_group`.
    let mut ten = Entries {
        bytes: unsafe { table_bytes(table + group * 0x258, 0x258) },
        size: 0x3C,
    };
    standings::rank_in_group(&mut ten, kept_name());
    Ok(())
}

/// What `rank_in_group` writes.
pub fn rank_regions() -> Vec<crate::service::harness::Region> {
    let table: u32 = unsafe { game_read(BEST_SCORES_RVA) };
    vec![(table, 5 * 0x258), (game_addr(KEPT_NAME_RVA) as u32, 0x1F)]
}

/// The current player's (`dword_75598C`) best score entered at the bottom of `group`'s ten
/// (`dword_7556D4`, entry 9): +0x38 its score (the record's `score_at`), +4 the player and
/// 1, +0x14 1, +8 / +0xC its +4 / +8, +0x10 its group (+0x90), +0 its name pointer and
/// +0x18 the name's 0x1F bytes; then moved up its group. Refused for a current player past
/// 3 or missing, no name, no table or a group past 4.
pub fn enter_best(group: Option<u32>, score_at: u32) -> Result<(), BadInput> {
    let n: u32 = unsafe { game_read(rva(0x75598C)) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(n);
    let group = group.unwrap_or(get(p + 0x90));
    let table: u32 = unsafe { game_read(BEST_SCORES_RVA) };
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

/// Whether a group has one of players 1 .. 3 (the original's test: player 4 is not
/// looked for) among `first`'s entries, its group at `group_at`.
pub fn group_has_player(all: &[u32], group_at: u32, group: u32) -> bool {
    all.iter()
        .any(|&e| get(e + group_at) == group && (1..=3).contains(&get(e + 4)))
}

/// Standings table B's first 0xB4 entries sorted by the signed dword at `key`, as
/// `sort_table_a`: going up (a later entry not above an earlier one moves up) or, with
/// `down`, going down (a later entry not below one moves up). Refused without a table.
pub fn sort_table_b(key: u32, down: bool) -> Result<(), BadInput> {
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
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
    unsafe {
        crate::framework::game_write(SORT_I_RVA, SORTED - 1);
        crate::framework::game_write(SORT_J_RVA, SORTED);
    }
    Ok(())
}

/// What `sort_table_b` writes.
pub fn sort_regions_b() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    vec![
        (first, (SORTED * B_ENTRY) as usize),
        at(SORT_I_RVA),
        at(SORT_J_RVA),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
    ]
}

/// The thirteen parts the inspection looks at (engine, bodywork, exhaust, electrics,
/// lights, cooling, steering, clutch, suspension, differential, brakes, gearbox, tyres):
/// their index in the player's +0xD4 .. +0x108 and in the limits (`dword_75ABD8` ..) and
/// penalties (`dword_75AC48` ..), fourteen each; index 4 (+0xE4, `dword_75ABE8`) is not
/// looked at.
pub const INSPECTED: [u32; 13] = [0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13];
pub const LIMITS_RVA: usize = rva(0x75ABD8); // dword_75ABD8: 14
pub const PENALTIES_RVA: usize = rva(0x75AC48); // dword_75AC48: 14

/// Whether the player's part `k` (an `INSPECTED` index) is at or past its limit (signed).
pub fn past_limit(p: u32, k: u32) -> bool {
    let limit: i32 = unsafe { game_read(LIMITS_RVA + 4 * k as usize) };
    get(p + 0xD4 + 4 * k) as i32 >= limit
}
