//! `menu::run_list` (0x468202): calls every function of a list of function addresses that
//! ends with -1. A screen has four such lists, which its loop and its frame callback run.
//!
//! In:  esi = the list
//! Out: every register preserved
//! Each function is called with the registers the one before left, esi being the place in
//! the list; after each call the original goes on from the esi it got back. The functions
//! are the game's (hooked or not), so the Rust version calls them with `call_game`, and
//! shadow mode runs the original alone: running a list twice would run every function in it
//! twice. The self-test uses lists of stand-ins that note the call. 26 callers.
//! Refuses a list without an end in its first 256 entries.
//! Manual patch with a private copy (0x0F bytes, nothing to relocate): its loop comes back
//! to the second byte, inside what the patch overwrites.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::Regs;

const END: u32 = u32::MAX;
const LIST_LIMIT: u32 = 256;

// ---- Rust version ----

/// Calls one function of a list with the registers given.
pub type Call<'a> = &'a mut dyn FnMut(u32, &mut Regs);

unsafe fn list_entry(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// `entry_regs` are the registers at entry.
pub unsafe fn run_list(list: u32, entry_regs: &Regs, call: Call) -> Result<(), BadInput> {
    unsafe {
        if !(0..LIST_LIMIT).any(|n| list_entry(list.wrapping_add(4 * n)) == END) {
            return Err(BadInput("a function list without an end"));
        }
        let mut g = *entry_regs;
        g.esi = list;
        while list_entry(g.esi) != END {
            call(list_entry(g.esi), &mut g);
            g.esi = g.esi.wrapping_add(4);
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry_regs = *r;
    unsafe { run_list(r.esi, &entry_regs, &mut crate::menu::table_calls::call) }?;
    Ok(())
}
