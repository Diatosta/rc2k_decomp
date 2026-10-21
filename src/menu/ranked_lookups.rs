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

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{EAX, EDI, Regs, SelfTest};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};

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

fn found(e: u32) -> Left {
    Left {
        carry: Some(false),
        zero: Some(true),
        ..Left::nothing().with(EDI, e)
    }
}

fn rust_entry(r: &Regs) -> Result<Left, Refused> {
    Ok(found(find_ranked_entry(r.edi, r.eax, r.ebx)?))
}

fn rust_position(r: &Regs) -> Result<Left, Refused> {
    Ok(found(find_ranked_position(r.edi, r.eax)?))
}

fn rust_champ(r: &Regs) -> Result<Left, Refused> {
    Ok(found(find_champ_position(r.edi, r.eax)?))
}

fn rust_first(r: &Regs) -> Result<Left, Refused> {
    let time = first_overall_time(r.edi, r.eax, r.ebx == 1)?;
    Ok(Left {
        carry: Some(r.ebx == 0),
        zero: Some(r.ebx == 1),
        ..Left::nothing().with(EAX, time)
    })
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

fn test(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    which: u32,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let list = if which == 3 {
        champ_list()
    } else {
        test_list()
    };
    let base = list.as_ptr() as u32;
    let len = if which == 3 {
        CHAMP_ENTRY_LEN
    } else {
        ENTRY_LEN
    };
    let rust: &dyn Fn(&Regs) -> Result<Left, Refused> = match which {
        0 => &rust_entry,
        1 => &rust_position,
        3 => &rust_champ,
        _ => &rust_first,
    };
    let cases: Vec<(u32, u32, u32)> = match which {
        // (start entry, eax, ebx)
        // every search finds its entry (the original's fatal end is not run)
        0 => (1..=11)
            .flat_map(|p| [(0, p, 2), (0, p, 3)])
            .chain((2..=11).map(|p| (4, p, 3)))
            .collect(),
        1 => (1..=11)
            .map(|p| (0, p, 0))
            .chain((3..=11).map(|p| (6, p, 0)))
            .collect(),
        3 => (1..=12)
            .map(|p| (0, p, 0))
            .chain((4..=12).map(|p| (3, p, 0)))
            .collect(),
        _ => [0u32, 1, 3]
            .iter()
            .flat_map(|&s| [(s, 2, 1), (s, 2, 0), (s, 1, 1), (s, 0, 0)])
            .collect(),
    };
    for (start, eax, ebx) in cases {
        check_state(
            &mut t,
            &format!("from entry {start}, eax {eax}, ebx {ebx}"),
            call_original,
            Regs {
                eax,
                ebx,
                edi: base + start * len,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            rust,
        );
    }
    // nothing found: the fatal error (the Rust version alone)
    let none = match which {
        0 => find_ranked_entry(base, 13, 2),
        1 => find_ranked_position(base, 13),
        3 => find_champ_position(base, 13),
        _ => {
            // a list whose entries have no class
            let mut bare = vec![0u32; 2 * ENTRY_LEN as usize / 4];
            bare[ENTRY_LEN as usize / 4] = u32::MAX;
            first_overall_time(bare.as_ptr() as u32, 2, true)
        }
    };
    let wanted = [NOT_FOUND_LOGGED, NOT_FOUND, NO_CLASS, NOT_IN_CHAMP][which as usize];
    t.check(none == Err(wanted.into()), || {
        format!(
            "nothing found: rust {none:?}, not fatal error {:#x}",
            wanted.code
        )
    });
    drop(list);
    t.finish()
}

/// A hook for one look-up.
macro_rules! lookup {
    ($module:ident, $name:literal, $address:expr, $len:expr, $relocate:expr, $which:expr, $rust:ident) => {
        pub mod $module {
            use super::*;
            use crate::framework::{RawHook, hook_asm, rva};
            use std::sync::atomic::AtomicUsize;

            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

            pub static HOOK: RawHook = RawHook::new($name, rva($address), thunk, &ORIGINAL)
                .private_copy($len)
                .relocate_calls(&$relocate)
                .manual_patch()
                .self_test(self_test);

            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL
            );

            extern "C" fn dispatch(r: &mut Regs) {
                dispatch_state(&HOOK, r, call_original, &[], &$rust);
            }

            fn self_test() -> (usize, usize) {
                test(&HOOK, call_original, $which)
            }
        }
    };
}

lookup!(
    find_ranked_entry_hook,
    "menu::find_ranked_entry",
    0x4A2547,
    0x25,
    [0x1A, 0x1F],
    0,
    rust_entry
);
lookup!(
    find_ranked_position_hook,
    "menu::find_ranked_position",
    0x4A2584,
    0x16,
    [0x10],
    1,
    rust_position
);
lookup!(
    find_champ_position_hook,
    "menu::find_champ_position",
    0x4A5A85,
    0x13,
    [0xD],
    3,
    rust_champ
);
lookup!(
    first_overall_time_hook,
    "menu::first_overall_time",
    0x4A327A,
    0x2B,
    [0x17],
    2,
    rust_first
);

/// The three, for the registry.
pub static HOOKS: [&dyn crate::framework::GameHook; 4] = [
    &find_champ_position_hook::HOOK,
    &find_ranked_entry_hook::HOOK,
    &find_ranked_position_hook::HOOK,
    &first_overall_time_hook::HOOK,
];
