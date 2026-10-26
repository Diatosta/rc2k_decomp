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
//! ebx = the class (`find_ranked_entry`) or 1 for the rally's time
//! Out: the two finds: edi = the entry, the rest kept, zero set and carry clear (the last
//! compare's); `first_overall_time`: eax = the time, the rest kept (edi pushed and
//! popped), the flags of `cmp ebx, 1`
//!
//! Callers: the results pages (`screens::results_pages`). Every entry is compared before the end
//! mark is looked at (the first one too, as the original). Refuses (before anything) a search that
//! runs 0x400 entries without the end mark.

use crate::errors::{BadInput, FatalError, Refused};
use crate::memory::blocks::Place;

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

fn get(at: Place) -> Result<u32, crate::errors::BadInput> {
    at.read("a ranked entry's value")
}

/// From `e` on, the first entry `wanted` takes; at the end mark `none`.
fn search(
    e: Place,
    wanted: impl Fn(Place) -> Result<bool, BadInput>,
    none: FatalError,
) -> Result<Place, Refused> {
    search_by(e, ENTRY_LEN, wanted, none)
}

/// `search` in entries of `len` bytes (converted: `rc2k_core::menu::standings::search`).
fn search_by(
    e: Place,
    len: u32,
    wanted: impl Fn(Place) -> Result<bool, BadInput>,
    none: FatalError,
) -> Result<Place, Refused> {
    use rc2k_core::menu::standings::{Found, search};
    let at = |n: u32| e.add(n.wrapping_mul(len));
    let fault = crate::menu::event_flow::Fault::default();
    let found = search(|n| fault.get_at(at(n)), |n| fault.ok(wanted(at(n))), LIMIT);
    fault.check()?;
    match found {
        Found::At(n) => Ok(at(n)),
        Found::End => Err(none.into()),
        Found::NoEnd => Err(BadInput("a ranked list without its end in 0x400 entries").into()),
    }
}

/// `find_ranked_entry`'s Rust version: the entry of `class` at `position`.
pub fn find_ranked_entry(list: Place, position: u32, class: u32) -> Result<Place, Refused> {
    search(
        list,
        |e| Ok(get(e.add(0x1C))? == class && get(e.add(0x44))? == position),
        NOT_FOUND_LOGGED,
    )
}

/// `find_ranked_position`'s Rust version: the entry at `position`.
pub fn find_ranked_position(list: Place, position: u32) -> Result<Place, Refused> {
    search(list, |e| Ok(get(e.add(0x44))? == position), NOT_FOUND)
}

/// `find_champ_position`'s Rust version: the history's entry at `position`.
pub fn find_champ_position(list: Place, position: u32) -> Result<Place, Refused> {
    search_by(
        list,
        CHAMP_ENTRY_LEN,
        |e| Ok(get(e.add(0x70))? == position),
        NOT_IN_CHAMP,
    )
}

/// `first_overall_time`'s Rust version: the time (`rally` the +0x18, else the +0x14).
pub fn first_overall_time(list: Place, kind: u32, rally: bool) -> Result<u32, Refused> {
    let e = if kind == 2 {
        search(list, |e| Ok(get(e.add(0x1C))? != 0), NO_CLASS)?
    } else {
        list
    };
    Ok(get(e.add(if rally { 0x18 } else { 0x14 }))?)
}
