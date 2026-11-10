//! `menu::run_list` (0x468202): calls every function of a list of function addresses that
//! ends with -1. A screen has four such lists, which its loop and its frame callback run.
//!
//! Each function is called with the registers the one before left, esi being the place in the list;
//! after each call the original goes on from the esi it got back. Refuses a list without an end in
//! its first 256 entries.

use crate::errors::{BadInput, Refused};
use crate::menu::fn_list::FnList;
use crate::menu::owner::Menus;

const END: u32 = u32::MAX;
const LIST_LIMIT: usize = 256;

/// Runs the functions of `list` in order. They are the typed tables' screen functions
/// (`menu::table_calls::call_listed`), which take no register: the registers the original
/// hands each (esi the place in the list, the rest as the one before left them) are
/// nobody's (2026-10-24). Refuses a list without its end mark in `LIST_LIMIT` entries.
pub fn run_list(menus: &Menus, list: FnList) -> Result<(), Refused> {
    let entries = list.entries();
    let Some(end) = entries.iter().take(LIST_LIMIT).position(|e| e.get() == END) else {
        return Err(BadInput("a function list without an end").into());
    };
    for e in &entries[..end] {
        crate::menu::table_calls::call_listed(menus, e.get())?;
    }
    Ok(())
}
