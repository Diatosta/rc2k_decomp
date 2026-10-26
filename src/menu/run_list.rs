//! `menu::run_list` (0x468202): calls every function of a list of function addresses that
//! ends with -1. A screen has four such lists, which its loop and its frame callback run.
//!
//! In:  esi = the list
//! Out: every register preserved
//! Each function is called with the registers the one before left, esi being the place in the list;
//! after each call the original goes on from the esi it got back. Refuses a list without an end in
//! its first 256 entries.

use crate::errors::{BadInput, Refused};

const END: u32 = u32::MAX;
const LIST_LIMIT: u32 = 256;

// ---- Rust version ----

fn list_entry(at: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(at).read::<u32>("a game value")
}

/// Runs the functions of `list` in order. They are the typed tables' screen functions
/// (`menu::table_calls::call_listed`), which take no register: the registers the original
/// hands each (esi the place in the list, the rest as the one before left them) are
/// nobody's (2026-10-24).
/// Whether the list has its end mark within `LIST_LIMIT` entries.
fn has_end(list: u32) -> Result<bool, crate::errors::BadInput> {
    for n in 0..LIST_LIMIT {
        if list_entry(list.wrapping_add(4 * n))? == END {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn run_list(list: u32) -> Result<(), Refused> {
    if !has_end(list)? {
        return Err(BadInput("a function list without an end").into());
    }
    let mut at = list;
    while list_entry(at)? != END {
        crate::menu::table_calls::call_listed(list_entry(at)?)?;
        at = at.wrapping_add(4);
    }

    Ok(())
}
