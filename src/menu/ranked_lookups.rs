//! The look-ups of the championship's results pages in the ranked times (0x9C-byte
//! `RankedTime` entries, -1 ends them; +0x14 the stage's time, +0x18 the rally's, +0x1C the
//! class, +0x44 the position), each ending in a fatal error when it finds nothing:
//! `find_ranked_entry` (a class at a position; 0x8070A7F9), `find_ranked_position` (0x8070A7FF),
//! `find_champ_position` (in the championship's history, 0x78-byte `ChampEntry` entries;
//! 0x8070A7FC) and `first_overall_time` (the first entry with a class, else the first entry;
//! its rally or stage time; 0x8070A982).
//!
//! Every entry is compared before the end mark is looked at (the first one too). Refuses (before
//! anything) a search that runs 0x400 entries without the end mark.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::standings::{with_a, with_b};
use rc2k_core::saves::entries::BestTime;
use rc2k_core::saves::rankings::{HistoryEntry, Table, Walked};

const LIMIT: u32 = 0x400;

const NOT_FOUND_LOGGED: FatalError = FatalError {
    code: 0x8070_A7F9,
    message: "Position in GB_TimedHST Not Found (see Log)",
};
const NOT_FOUND: FatalError = FatalError {
    code: 0x8070_A7FF,
    message: "Position in GB_TimedHST Not Found",
};
const NOT_IN_CHAMP: FatalError = FatalError {
    code: 0x8070_A7FC,
    message: "Position in GB_ChampHST Not Found (see Log)",
};
const NO_CLASS: FatalError = FatalError {
    code: 0x8070_A982,
    message: "Seriously buggered",
};

/// From a table's first entry on, the first entry `wanted` takes (a copy); at the end mark
/// `none` (converted: `rc2k_core::menu::standings::search`).
fn search<T: Walked + Copy + Default>(
    t: &Table<T>,
    wanted: impl Fn(&T) -> bool,
    none: FatalError,
) -> Result<T, Refused> {
    use rc2k_core::menu::standings::{Found, search};
    let found = search(
        |n| t.first_at(n as usize).unwrap_or(u32::MAX),
        |n| t.entries.get(n as usize).is_some_and(&wanted),
        LIMIT,
    );
    match found {
        Found::At(n) => Ok(t.entries[n as usize]),
        Found::End => Err(none.into()),
        Found::NoEnd => Err(BadInput("a ranked list without its end in 0x400 entries").into()),
    }
}

/// Table A's entry of `class` at `position`.
pub fn find_ranked_entry(
    position: u32,
    class: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<BestTime, Refused> {
    with_a(tables, |t| {
        search(
            t,
            |e| e.head.group == class && e.head.figure == position,
            NOT_FOUND_LOGGED,
        )
    })?
}

/// Table A's entry at `position`.
pub fn find_ranked_position(
    position: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<BestTime, Refused> {
    with_a(tables, |t| {
        search(t, |e| e.head.figure == position, NOT_FOUND)
    })?
}

/// The history's entry at `position`.
pub fn find_champ_position(
    position: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<HistoryEntry, Refused> {
    with_b(tables, |t| {
        search(t, |e| e.position == position, NOT_IN_CHAMP)
    })?
}

/// Table A's time (`rally` the total, else the time).
pub fn first_overall_time(
    kind: u32,
    rally: bool,
    tables: &crate::saves::best_times::BestTables,
) -> Result<u32, Refused> {
    let e = if kind == 2 {
        with_a(tables, |t| search(t, |e| e.head.group != 0, NO_CLASS))??
    } else {
        with_a(tables, |t| t.entries[0])?
    };
    Ok(if rally { e.head.rally_time } else { e.head.key })
}
