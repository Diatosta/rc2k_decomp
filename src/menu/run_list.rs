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
use crate::framework::{Regs, SelfTest};
use std::sync::atomic::{AtomicU32, Ordering};

const END: u32 = u32::MAX;
const LIST_LIMIT: u32 = 256;

// ---- Rust version ----

/// Calls one function of a list with the registers given.
pub type Call<'a> = &'a mut dyn FnMut(u32, &mut Regs);

pub fn in_game(function: u32, regs: &mut Regs) {
    crate::menu::table_calls::call_any(function, regs);
}

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

static NOTED: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
static NOTE_COUNT: AtomicU32 = AtomicU32::new(0);

/// A stand-in: notes its number and the esi it was called with (its place in the list).
macro_rules! stand_in {
    ($name:ident, $number:literal) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push ecx",
                "mov ecx, [{count}]",
                "and ecx, 3",
                "mov dword ptr [{noted} + ecx * 8], {number}",
                "mov [{noted} + ecx * 8 + 4], esi",
                "inc dword ptr [{count}]",
                "pop ecx",
                "ret",
                count = sym NOTE_COUNT,
                noted = sym NOTED,
                number = const $number,
            )
        }
    };
}
stand_in!(first, 1);
stand_in!(second, 2);
stand_in!(third, 3);

fn noted() -> ([u32; 8], u32) {
    (
        NOTED.each_ref().map(|n| n.load(Ordering::SeqCst)),
        NOTE_COUNT.load(Ordering::SeqCst),
    )
}

fn clear_notes() {
    for n in NOTED.iter().chain([&NOTE_COUNT]) {
        n.store(0, Ordering::SeqCst);
    }
}

/// The lists the tests run: none, one and three functions.
pub fn test_lists() -> [Vec<u32>; 3] {
    let f = |function: unsafe extern "C" fn()| function as *const () as u32;
    [
        vec![END],
        vec![f(second), END],
        vec![f(first), f(third), f(second), END],
    ]
}

pub fn check(
    t: &mut SelfTest,
    what: &str,
    before: Regs,
    original: &dyn Fn(&mut Regs),
    rust: &dyn Fn(&Regs) -> Result<(), BadInput>,
) {
    clear_notes();
    let mut ro = before;
    original(&mut ro);
    let theirs = noted();
    clear_notes();
    let outcome = rust(&before);
    let mine = noted();
    let preserved = ro.preserved_except(&before, 0);
    t.check(outcome.is_ok() && theirs == mine && preserved, || {
        format!("{what}: original {theirs:x?} preserved={preserved}; rust {outcome:?} {mine:x?}")
    });
}
