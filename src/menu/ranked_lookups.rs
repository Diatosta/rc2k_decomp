//! The look-ups of the championship's results pages in the ranked times (`g_rankedTimes`,
//! 0x9C-byte `RankedTime` entries, -1 ends them; +0x14 the stage's time, +0x18 the
//! rally's, +0x1C the class, +0x44 the position), each ending in the original's fatal error
//! when it finds nothing:
//! - `menu::find_ranked_entry` (0x4A2547): from edi on, the entry of class ebx at position
//!   eax; none: (ebx 1, `sub_4A1F66`, which does nothing) fatal error 0x8070A7F9 "Position in
//!   GB_TimedHST Not Found (see Log)";
//! - `menu::find_ranked_position` (0x4A2584): from edi on, the entry at position eax; none:
//!   fatal error 0x8070A7FF "Position in GB_TimedHST Not Found";
//! - `menu::find_champ_position` (0x4A5A85): from edi on, in the championship's history
//!   (`g_champHistory`, 0x78-byte `ChampEntry` entries), the entry at position eax (+0x70);
//!   none: fatal error 0x8070A7FC "Position in GB_ChampHST Not Found (see Log)";
//! - `menu::first_overall_time` (0x4A327A): for eax 2 the first entry from edi on with a
//!   class (+0x1C not 0; none: fatal error 0x8070A982 "Seriously buggered"), else the entry
//!   at edi; its rally time (ebx 1) or its stage time.
//!
//! In:  edi = the entry to start from, eax = the position (the kind for `first_overall_time`),
//!      ebx = the class (`find_ranked_entry`) or 1 for the rally's time
//! Out: the two finds: edi = the entry, the rest kept, zero set and carry clear (the last
//!      compare's); `first_overall_time`: eax = the time, the rest kept (edi pushed and
//!      popped), the flags of `cmp ebx, 1`
//!
//! Callers: the results pages (`screens::results_pages`). Every entry is compared before
//! the end mark is looked at (the first one too, as the original). Refuses (before
//! anything) a search that runs 0x400 entries without the end mark. Each loops back to its
//! first byte or has a short jump in its first five: manual patches with private copies,
//! their calls and jumps out relocated. Compared in shadow mode. Self-test: the original
//! against the Rust version on a list of the test's: each position, each class, the first
//! entry with no class and with one; the Rust version alone on a search that finds nothing
//! (the fatal error).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};

pub const ENTRY_LEN: u32 = 0x9C;
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
pub const CHAMP_ENTRY_LEN: u32 = 0x78;
const NO_CLASS: FatalError = FatalError {
    code: 0x8070_A982,
    message: "Seriously buggered",
};

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// From `e` on, the first entry `wanted` takes; at the end mark `none`.
fn search(e: u32, wanted: impl Fn(u32) -> bool, none: FatalError) -> Result<u32, Refused> {
    search_by(e, ENTRY_LEN, wanted, none)
}

/// `search` in entries of `len` bytes (converted: `rc2k_core::menu::standings::search`).
fn search_by(
    e: u32,
    len: u32,
    wanted: impl Fn(u32) -> bool,
    none: FatalError,
) -> Result<u32, Refused> {
    use rc2k_core::menu::standings::{Found, search};
    let at = |n: u32| e.wrapping_add(n.wrapping_mul(len));
    match search(|n| get(at(n)), |n| wanted(at(n)), LIMIT) {
        Found::At(n) => Ok(at(n)),
        Found::End => Err(none.into()),
        Found::NoEnd => Err(BadInput("a ranked list without its end in 0x400 entries").into()),
    }
}

/// `find_ranked_entry`'s Rust version: the entry of `class` at `position`.
pub fn find_ranked_entry(list: u32, position: u32, class: u32) -> Result<u32, Refused> {
    search(
        list,
        |e| get(e + 0x1C) == class && get(e + 0x44) == position,
        NOT_FOUND_LOGGED,
    )
}

/// `find_ranked_position`'s Rust version: the entry at `position`.
pub fn find_ranked_position(list: u32, position: u32) -> Result<u32, Refused> {
    search(list, |e| get(e + 0x44) == position, NOT_FOUND)
}

/// `find_champ_position`'s Rust version: the history's entry at `position`.
pub fn find_champ_position(list: u32, position: u32) -> Result<u32, Refused> {
    search_by(
        list,
        CHAMP_ENTRY_LEN,
        |e| get(e + 0x70) == position,
        NOT_IN_CHAMP,
    )
}

/// `first_overall_time`'s Rust version: the time (`rally` the +0x18, else the +0x14).
pub fn first_overall_time(list: u32, kind: u32, rally: bool) -> Result<u32, Refused> {
    let e = if kind == 2 {
        search(list, |e| get(e + 0x1C) != 0, NO_CLASS)?
    } else {
        list
    };
    Ok(get(e + if rally { 0x18 } else { 0x14 }))
}

// ---- self-tests ----

/// A list of 24 entries (the first two with no class and position 99, then classes 2 and 3
/// in turn, positions 1 .. 11 in each; times of the test's) and its end mark.
pub fn test_list() -> Vec<u32> {
    let words = ENTRY_LEN as usize / 4;
    let mut list = vec![0u32; 25 * words];
    for n in 0..24usize {
        let w = &mut list[n * words..(n + 1) * words];
        w[0] = n as u32;
        w[0x14 / 4] = 5000 + 311 * n as u32;
        w[0x18 / 4] = 9000 + 577 * n as u32;
        w[0x1C / 4] = if n < 2 { 0 } else { [2, 3][n % 2] };
        w[0x44 / 4] = if n < 2 { 99 } else { (n as u32 - 2) / 2 + 1 };
    }
    list[24 * words] = u32::MAX;
    list
}

/// A championship history of the test's: 12 entries, positions 1 .. 12 (in another order),
/// and its end mark.
pub fn champ_list() -> Vec<u32> {
    let words = CHAMP_ENTRY_LEN as usize / 4;
    let mut list = vec![0u32; 13 * words];
    for n in 0..12usize {
        let w = &mut list[n * words..(n + 1) * words];
        w[0] = n as u32;
        w[0x70 / 4] = [3u32, 1, 2, 6, 4, 5, 9, 7, 8, 12, 10, 11][n];
    }
    list[12 * words] = u32::MAX;
    list
}
