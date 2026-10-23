//! `marks::tidy_marks` (0x4D146D): the marks' memory given back at a stage's unload. Its
//! steps, each a call (the nullsubs left out): `sub_4D06F3` (for each of the settings'
//! +0x5C cars from the first, its list at +0x2E8 freed and its +0x328 made
//! `g_marksTaken`), then `memory::free_list` of `g_marksRingList` (`sub_4CFB3D`),
//! `stru_935620` (`sub_4CB5A8`), `g_marksBMaskList` and `stru_934D76` (`sub_4C8F4D`),
//! `g_marksPlacesList` (`sub_4C38F5`) and `g_marksLinksList` (`sub_4C4E22` ->
//! `sub_4C3EE3` -> `sub_4C5409`). Those steps are not hooked of their own: each is a
//! `lea ebx` and a call.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: eax the cars' count, ebx `g_marksLinksList`, ebp the last car's +0x29C (the
//!      settings when there is none); the rest kept
//!
//! `sub_4D06F3` tests `memory::free_list`'s carry (`jb`), which is always clear (its last
//! flag-setting instruction is a `test`), so its other path (a `push` left on the stack
//! and a count of 0x1FFFFFFF: section 16) is never taken. Caller: the stage's unload
//! (0x4449AE). It begins with its call: a private copy (0x47 bytes, its 14 calls
//! relocated) with a manual patch. Shadow mode runs the original alone (it frees).
//! Refuses (before anything) null settings, a cars' count below 0 or above 8 (the
//! original's loop runs until it meets the count), and the game's own heap with a list
//! that holds blocks (`free_list`'s refusal). Self-test: with every list empty in the game,
//! blocks of the test's put in each, freed by the original and by the Rust version; the
//! lists, the memory counters, the cars' +0x328 and the registers compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::marks::lines::{SETTINGS_RVA, TAKEN_RVA};
use crate::memory::free_list::free_list;
use crate::memory::{MemoryList, OWN_HEAP_RVA};

const CAR_LEN: u32 = 0x4C8;
const CAR_PART: u32 = 0x29C; // `sub_4D06F3`'s ebp: the car's +0x29C
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
const LISTS_RVA: [usize; 6] = [
    rva(0x93A580), // g_marksRingList
    rva(0x935620), // stru_935620
    rva(0x934DBA), // g_marksBMaskList
    rva(0x934D76), // stru_934D76
    rva(0x8DB570), // g_marksPlacesList
    rva(0x9206F0), // g_marksLinksList
];
/// Each car's list, worked out without writing (none for a count past the refusals).
unsafe fn car_lists(settings: u32) -> Vec<u32> {
    unsafe {
        let count = (*((settings + 0x5C) as *const i32)).clamp(0, MOST_CARS) as u32;
        (0..count)
            .map(|n| settings + n * CAR_LEN + CAR_PART + 0x4C)
            .collect()
    }
}

/// Every list it frees, the cars' first.
unsafe fn all_lists(settings: u32) -> Vec<u32> {
    unsafe {
        let mut r = car_lists(settings);
        r.extend(LISTS_RVA.map(|a| game_addr(a) as u32));
        r
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn tidy_marks(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let count = *((settings + 0x5C) as *const i32);
        if !(0..=MOST_CARS).contains(&count) {
            return Err(BadInput("a cars' count below 0 or above 8").into());
        }
        if game_read::<u32>(OWN_HEAP_RVA) == 1
            && all_lists(settings)
                .iter()
                .any(|&l| (*(l as *const MemoryList)).head != 0)
        {
            return Err(
                BadInput("the game's own heap is in use with a list holding blocks").into(),
            );
        }
        g.ebp = settings;
        for n in 0..count as u32 {
            g.ebp = settings + n * CAR_LEN + CAR_PART;
            g.ebx = g.ebp + 0x4C;
            free_list(g.ebx as *mut MemoryList)?;
            ((g.ebp + 0x8C) as *mut u32).write_unaligned(game_read(TAKEN_RVA));
        }
        g.eax = count as u32;
        for a in LISTS_RVA {
            g.ebx = game_addr(a) as u32;
            free_list(g.ebx as *mut MemoryList)?;
        }
    }
    Ok(g)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { tidy_marks(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
